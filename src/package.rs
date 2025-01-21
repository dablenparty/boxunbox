use std::{iter, path::PathBuf, sync::LazyLock};

use anyhow::Context;
use regex::Regex;

use crate::{cli::BoxUnboxCli, rc::BoxUnboxRcArgs, PackageEntry};

#[derive(Debug, Clone)]
pub struct PackageOptions {
    /// Package to `box` or `unbox`. Can be a single file or a directory.
    pub package: PathBuf,
    /// Target directory where the symlinks are stored. Must be a directory.
    pub target: PathBuf,
    /// Include directories.
    pub include_dirs: bool,
    /// Ignore file names by regex.
    pub ignore_pats: Vec<Regex>,
}

impl TryFrom<BoxUnboxCli> for PackageOptions {
    type Error = anyhow::Error;

    fn try_from(value: BoxUnboxCli) -> Result<Self, Self::Error> {
        Ok(Self {
            package: value.package,
            target: value
                .target
                .context("no target specified but is required")?,
            include_dirs: value.include_dirs.unwrap_or(false),
            ignore_pats: value.ignore_pats,
        })
    }
}

impl PackageOptions {
    /// Create a new [`PackageOptions`] from [`BoxUnboxCli`] and [`BoxUnboxRcArgs`] by merging
    /// the two structs. CLI args are preferred over RC args.
    ///
    /// # Arguments
    ///
    /// - `cli` - CLI args
    /// - `rc` - RC file args
    pub fn from_parts(cli: BoxUnboxCli, rc: BoxUnboxRcArgs) -> Self {
        let mut ignore_pats = rc.ignore_pats;
        ignore_pats.extend(cli.ignore_pats);

        Self {
            package: cli.package,
            target: cli.target.unwrap_or(rc.target),
            include_dirs: cli.include_dirs.unwrap_or(rc.include_dirs),
            ignore_pats,
        }
    }

    /// Gets all entries to either `box` or `unbox` from a package.
    ///
    /// # Arguments
    ///
    /// - `package_opts` - Package options
    ///
    /// # Errors
    ///
    /// An error is returned if one occurs when reading the package directory. Errors for individual
    /// [`DirEntry`]'s do not end this function and are instead collected into the returned `Vec`.
    pub fn get_package_entries(&self) -> anyhow::Result<Vec<anyhow::Result<PackageEntry>>> {
        static RC_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("\\.unboxrc$").expect(".unboxrc regex failed to compile"));

        let PackageOptions {
            package,
            include_dirs,
            ignore_pats,
            ..
        } = self;

        anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

        let pkg_entries = package
            .read_dir()
            .with_context(|| format!("Failed to read directory {package:?}"))?
            .filter_map(|res| {
                if let Ok(ref ent) = res {
                    // need utf8 string for regex
                    let file_name = ent.file_name();
                    // shadow previous name to get around temp value error without keeping it
                    let file_name = file_name.to_string_lossy();

                    if ignore_pats
                        .iter()
                        .chain(iter::once(&RC_REGEX.clone()))
                        .any(|re| re.is_match(&file_name))
                        || (!include_dirs && ent.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                    {
                        return None;
                    }
                }
                Some(
                    res.map(PackageEntry::from)
                        .context("Failed to read dir entry"),
                )
            })
            .collect();

        Ok(pkg_entries)
    }
}
