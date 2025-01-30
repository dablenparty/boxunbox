use std::{
    fs, io, iter,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use errors::{ParseError, UnboxError, WriteError};
use regex::Regex;
use ron::ser::PrettyConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::{
    cli::BoxUnboxCli,
    constants::BASE_DIRS,
    utils::{expand_into_pathbuf, os_symlink},
};

pub mod errors;

/// Utility function to deserialize a [`PathBuf`] while expanding environment variables and `~`.
///
/// # Arguments
///
/// - `d` - Argument to deserialize, expected to be `&str`.
fn __de_pathbuf<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(d)?;
    expand_into_pathbuf(s).map_err(D::Error::custom)
}

/// Utility function returning the default value for [`PackageConfig::target`], which is the users
/// home directory.
fn __target_default() -> PathBuf {
    BASE_DIRS.home_dir().to_path_buf()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageConfig {
    #[serde(skip)]
    pub package: PathBuf,
    #[serde(skip)]
    pub dry_run: bool,

    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
    #[serde(with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
}

impl TryFrom<BoxUnboxCli> for PackageConfig {
    type Error = io::Error;

    fn try_from(value: BoxUnboxCli) -> Result<Self, Self::Error> {
        let BoxUnboxCli {
            package,
            target,
            ignore_pats,
            dry_run,
        } = value;
        let conf = Self {
            package: package.canonicalize()?,
            target: target.unwrap_or_else(__target_default).canonicalize()?,
            ignore_pats,
            dry_run,
        };
        Ok(conf)
    }
}

impl PackageConfig {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Merge with [`BoxUnboxCli`] args. Consumes both this struct and the `cli` args.
    ///
    /// # Arguments
    ///
    /// - `cli` - CLI args to merge with.
    pub fn merge_with_cli(self, cli: BoxUnboxCli) -> io::Result<Self> {
        let mut ignore_pats = self.ignore_pats;
        ignore_pats.extend(cli.ignore_pats);

        let conf = Self {
            package: self.package,
            target: cli.target.unwrap_or(self.target).canonicalize()?,
            dry_run: cli.dry_run,
            ignore_pats,
        };

        Ok(conf)
    }

    /// Try to parse [`PackageConfig`] from a given package path.
    ///
    /// # Arguments
    ///
    /// - `package` - Package [`Path`].
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - The RC file doesn't exist.
    /// - Failure to read RC file.
    /// - Failure to parse RC file with [`ron`].
    pub fn try_from_package<P: AsRef<Path>>(package: P) -> Result<Self, ParseError> {
        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        if !rc_file.exists() {
            return Err(ParseError::FileNotFound(rc_file));
        }

        let rc_str = fs::read_to_string(&rc_file)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let mut rc: PackageConfig = ron::from_str(&rc_str)?;
        rc.package = package.to_path_buf();

        Ok(rc)
    }

    /// Save this [`PackageConfig`] to a `package` directory.
    ///
    /// # Arguments
    ///
    /// - `package` - Package to save this config to.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - This struct fails to serialize into RON.
    /// - The file cannot be created/written to
    pub fn save_to_package<P: AsRef<Path>>(&self, package: P) -> Result<(), WriteError> {
        let mut clone_self = self.clone();
        let home_dir = BASE_DIRS.home_dir();

        if let Ok(path) = clone_self.target.strip_prefix(home_dir) {
            clone_self.target = PathBuf::from("~/").join(path);
        }

        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        // TODO: do something if the config already exists, maybe an error?
        // WARN: this overwrites the existing file, be careful!
        let ron_str =
            ron::ser::to_string_pretty(&clone_self, PrettyConfig::new().struct_names(true))?;
        fs::write(rc_file, ron_str)?;

        Ok(())
    }

    /// Unbox the `package` defined by this [`PackageConfig`] into the `target` defined by the
    /// same. Unboxing is done by either symlinking the directory itself if the target doesn't
    /// exist or by iterating over each directory entry and linking each. A more advanced algorithm
    /// may be implemented at a later date.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - The `package` does not or cannot be verified to exist.
    /// - The symlink cannot be created.
    pub fn unbox(&self) -> Result<(), UnboxError> {
        static RC_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("\\.unboxrc").unwrap());

        let PackageConfig {
            package,
            target,
            ignore_pats,
            dry_run,
        } = self;

        if !package
            .try_exists()
            .with_context(|| format!("failed to check existence of package: {package:?}"))?
        {
            return Err(UnboxError::PackageNotFound(package.clone()));
        }

        // FIXME: currently, if file matches an ignore pattern but one of its parent directory's
        // is already linked, the file ends up NOT being ignored since the parent folder is linked.
        // Every day, the "only files" option seems better and better...
        let ignore_pats = ignore_pats
            .iter()
            .chain(iter::once(&*RC_REGEX))
            .collect::<Vec<&Regex>>();

        /* TODO: different algorithms
        - only files: instead of linking directories, create them at the target path and
                           link their files instead.
        */
        // currently tries to create as few links as possible

        // essentially guards against errors; if even ONE occurs, abort and return it.
        let pkg_entry_paths = walkdir::WalkDir::new(package)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|res| match res {
                Ok(ent) => {
                    let file_name = ent.file_name().to_string_lossy();
                    if ignore_pats.iter().any(|pat| pat.is_match(&file_name)) {
                        None
                    } else {
                        Some(Ok(ent.path().to_path_buf()))
                    }
                }
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<PathBuf>, _>>()?;

        let mut linked_dirs = Vec::new();
        let mut planned_links = Vec::new();

        // /path/to/package/entry
        for path in &pkg_entry_paths {
            // if a parent directory is linked, don't link the file as that'd be circular
            if linked_dirs.iter().any(|d| path.strip_prefix(d).is_ok()) {
                continue;
            }
            // entry
            let stripped = path.strip_prefix(package).unwrap_or_else(|err| {
                unreachable!(
                    "failed to strip package prefix '{package:?}' from package entry '{path:?}': {err:?}"
                )
            });
            // /path/to/target/entry
            let new_target = target.join(stripped);
            // if the target entry exists and is a directory, just skip it. otherwise
            // return an error.
            if new_target
                .try_exists()
                .with_context(|| format!("failed to verify existence of {new_target:?}"))?
            {
                if new_target.is_dir() {
                    continue;
                }

                // exists, but is file/symlink
                return Err(UnboxError::TargetAlreadyExists {
                    package_entry: path.clone(),
                    target_entry: new_target.clone(),
                });
            }

            planned_links.push((path, new_target));
            if path.is_dir() {
                linked_dirs.push(path);
            }
        }

        if *dry_run {
            // TODO: better dry run output (colors?)
            for (src, dest) in planned_links {
                println!("{} -> {}", src.display(), dest.display());
            }
        } else {
            planned_links.into_iter().try_for_each(|(src, dest)| {
                os_symlink(src, &dest)
                    .with_context(|| format!("failed to symlink {src:?} -> {dest:?}"))
            })?;
        }

        Ok(())
    }
}
