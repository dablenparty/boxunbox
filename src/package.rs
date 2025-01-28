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
}
