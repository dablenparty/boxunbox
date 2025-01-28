use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use errors::{ParseError, UnboxError, WriteError};
use ron::ser::PrettyConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::{cli::BoxUnboxCli, constants::BASE_DIRS, expand_into_pathbuf};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageConfig {
    #[serde(skip)]
    pub package: PathBuf,
    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
}

impl PackageConfig {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Make a new [`PackageConfig`] with a given [`Path`] `p` and default options.
    ///
    /// # Arguments
    ///
    /// - `p` - Package to make config for
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        Self {
            package: p.as_ref().to_path_buf(),
            // use the ~ instead of the absolute path for simple multi-system support
            target: PathBuf::from("~/"),
        }
    }

    /// Merge with [`BoxUnboxCli`] args. Consumes both this struct and the `cli` args.
    ///
    /// # Arguments
    ///
    /// - `cli` - CLI args to merge with.
    pub fn merge_with_cli(self, cli: BoxUnboxCli) -> Self {
        Self {
            package: cli.package,
            target: cli.target.unwrap_or(self.target),
        }
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

        let rc = ron::from_str(&rc_str)?;

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
        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        // TODO: do something if the config already exists, maybe an error?
        // WARN: this overwrites the existing file, be careful!
        let ron_str = ron::ser::to_string_pretty(self, PrettyConfig::new().struct_names(true))?;
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
        let PackageConfig { package, target } = self;
        let link_path = target.join(PackageConfig::__rc_file_name());

        if !package
            .try_exists()
            .with_context(|| format!("failed to check existence of package: {link_path:?}"))?
        {
            return Err(UnboxError::PackageNotFound(package.clone()));
        }

        /* TODO: different algorithms
        - only files: instead of linking directories, create them at the target path and
                           link their files instead.
        - least links: like stow, create the fewest links possible (files & folders)
        */

        // std::fs::soft_link works fine, but is weird on Windows. The documentation recommends
        // using OS-specific libraries to make intent explicit.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(package, &link_path).with_context(|| {
                format!("failed creating soft link: {package:?} -> {link_path:?}")
            })?;
        }

        #[cfg(all(not(unix), windows))]
        {
            todo!("PackageConfig::unbox for Windows")
        }

        #[cfg(not(any(windows, unix)))]
        {
            unimplemented!()
        }

        Ok(())
    }
}
