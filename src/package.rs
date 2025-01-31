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
            ..
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
    pub fn merge_with_cli(self, cli: &BoxUnboxCli) -> io::Result<Self> {
        let mut ignore_pats = self.ignore_pats;
        ignore_pats.extend(cli.ignore_pats.clone());

        let conf = Self {
            package: self.package,
            target: cli.target.clone().unwrap_or(self.target).canonicalize()?,
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

        let do_dry_run = self.dry_run;
        let root_package = self.package.clone();
        match root_package.try_exists() {
            Ok(true) => {}
            Ok(false) | Err(_) => return Err(UnboxError::PackageNotFound(root_package.clone())),
        }

        // TODO: least links algorithm?
        // At the very least, I want a way to choose between the algorithms

        /*
        NOTE: currently only creates file symlinks, not directories
        I chose this because I had issues where the directory would get linked, then files
        placed there by other programs would show up in the original location, which I don't
        want.
        */

        let mut config_stack = vec![self.clone()];

        /// Utility macro that expands to get the last config off the config stack.
        macro_rules! clone_last_config {
            () => {
                config_stack
                    .last()
                    .expect("there should be at least one config in the stack")
                    .clone()
            };
        }

        // essentially guards against errors; if even ONE occurs, abort and return it.
        let pkg_entry_paths = walkdir::WalkDir::new(&root_package)
            .sort_by_file_name()
            .into_iter()
            .skip(1) // skip root package
            .map(|res| res.map(|ent| ent.path().to_path_buf()))
            .collect::<Result<Vec<PathBuf>, _>>()?;

        // plan your moves first before doing anything in case something fails; don't want to get
        // halfway done unboxing just to realize you have to box it all back up!
        // TODO: rollback plans on error (consider a plan struct)
        let mut planned_links = Vec::new();
        let mut planned_dirs = Vec::new();

        for path in pkg_entry_paths {
            let path_is_dir = path.is_dir();

            let last_config = clone_last_config!();

            // If we're in a subdir of the last config, keep using it. Otherwise, pop it off the
            // stack and get the next one.
            let last_config = if path.starts_with(&last_config.package) {
                last_config
            } else {
                let _ = config_stack
                    .pop()
                    .expect("there should be at least one config in the stack");
                clone_last_config!()
            };

            // read the config of this subdir
            // if the config exists, add it to the stack; if not, don't care
            match PackageConfig::try_from_package(&path) {
                Ok(config) => config_stack.push(config),
                Err(ParseError::FileNotFound(_)) => {}
                Err(err) => return Err(err.into()),
            }

            let file_name = path
                .file_name()
                .unwrap_or_else(|| path.as_os_str())
                .to_string_lossy();

            // check all ignore patterns in the stack
            if config_stack
                .iter()
                .flat_map(|conf| conf.ignore_pats.as_slice())
                .chain(iter::once(&*RC_REGEX))
                .any(|re| re.is_match(&file_name))
            {
                #[cfg(debug_assertions)]
                println!("ignoring file {path:?}");

                continue;
            }

            let PackageConfig {
                package, target, ..
            } = last_config;

            // /path/to/package/entry -> /entry
            let stripped = path.strip_prefix(&package).unwrap_or_else(|err| {
                unreachable!(
                    "failed to strip package prefix '{package:?}' from package entry '{path:?}': {err:?}"
                )
            });
            // /entry -> /path/to/target/entry
            let new_target = target.join(stripped);

            // if the target exists, is a directory, and `path_is_dir`, just skip it; otherwise,
            // return an error.
            if new_target
                .try_exists()
                .with_context(|| format!("failed to verify existence of {new_target:?}"))?
            {
                // if both the original and target are already directories
                if path_is_dir && new_target.is_dir() {
                    continue;
                }

                // exists, but is file/symlink
                return Err(UnboxError::TargetAlreadyExists {
                    package_entry: path.clone(),
                    target_entry: new_target.clone(),
                });
            }

            if path_is_dir {
                planned_dirs.push(new_target);
            } else {
                planned_links.push((path, new_target));
            }
        }

        if do_dry_run {
            // TODO: better dry run output (colors?)
            for dir in planned_dirs {
                println!("mkdir {}", dir.display());
            }
            for (src, dest) in planned_links {
                println!("{} -> {}", src.display(), dest.display());
            }
        } else {
            // make directories first, then link target files
            planned_dirs.into_iter().try_for_each(|dir| {
                fs::create_dir(&dir).with_context(|| format!("failed to mkdir {dir:?}"))
            })?;
            planned_links.into_iter().try_for_each(|(src, dest)| {
                os_symlink(&src, &dest)
                    .with_context(|| format!("failed to symlink {src:?} -> {dest:?}"))
            })?;
        }

        Ok(())
    }
}
