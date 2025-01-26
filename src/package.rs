use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::expand_into_pathbuf;

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

/// Utility function returning the default value for [`BoxUnboxRc::target`], which is the users
/// home directory.
fn __target_default() -> PathBuf {
    directories_next::UserDirs::new()
        .expect("failed to locate user home directory")
        .home_dir()
        .to_path_buf()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageConfig {
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
            target: __target_default(),
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
    pub fn try_from_package<P: AsRef<Path>>(package: P) -> Result<Self, errors::ParseError> {
        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        if !rc_file.exists() {
            return Err(errors::ParseError::FileNotFound(rc_file));
        }

        let rc_str = fs::read_to_string(&rc_file)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let rc = ron::from_str(&rc_str)?;

        Ok(rc)
    }
}
