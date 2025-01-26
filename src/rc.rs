use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ron::ser::PrettyConfig;
use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};

use crate::{expand_into_pathbuf, package::PackageConfig};

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

fn __target_default() -> PathBuf {
    expand_into_pathbuf("~/").expect("failed to expand a single `~`")
}

#[derive(Debug, thiserror::Error)]
pub enum RcParseError {
    /// The RC file doesn't exist
    #[error("no RC file found at `{0}`")]
    RcFileNotFound(PathBuf),
    /// Failed to read the RC file
    #[error("failed to read rc file: {0}")]
    RcFileFailedToRead(#[from] anyhow::Error),
    /// Failed to parse RC file as RON
    #[error("failed to parse rc file: {0}")]
    BadRcFormat(#[from] ron::error::SpannedError),
}

#[derive(Debug, thiserror::Error)]
pub enum RcSaveError {
    #[error("failed to serialize RC struct: {0}")]
    RcFailedToSerialize(#[from] ron::Error),
    #[error("failed to write rc file: {0}")]
    RcFailedToWrite(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BoxUnboxRc {
    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
}

impl Default for BoxUnboxRc {
    fn default() -> Self {
        Self {
            target: __target_default(),
        }
    }
}

impl From<PackageConfig> for BoxUnboxRc {
    fn from(value: PackageConfig) -> Self {
        Self::from(&value)
    }
}

impl From<&PackageConfig> for BoxUnboxRc {
    fn from(value: &PackageConfig) -> Self {
        Self {
            target: value.target.clone(),
        }
    }
}

impl BoxUnboxRc {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Try to parse [`BoxUnboxRc`] args from a given package path.
    ///
    /// # Arguments
    ///
    /// - `p` - Package [`Path`].
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - The RC file doesn't exist.
    /// - Failure to read RC file.
    /// - Failure to parse RC file with [`ron`].
    pub fn try_parse_from_package<P: AsRef<Path>>(p: P) -> Result<Self, RcParseError> {
        let package = p.as_ref();
        let rc_file = package.join(BoxUnboxRc::__rc_file_name());

        if !rc_file.exists() {
            return Err(RcParseError::RcFileNotFound(rc_file.to_owned()));
        }

        let rc_str = fs::read_to_string(&rc_file)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let rc = ron::from_str(&rc_str)?;

        Ok(rc)
    }

    /// Save these RC args to a `package`.
    ///
    /// # Arguments
    ///
    /// - `package` - The `package` to save these args to.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - These args fail to serialize.
    /// - The RC file cannot be written to.
    pub fn save_package_rc<P: AsRef<Path>>(&self, package: P) -> Result<(), RcSaveError> {
        let package = package.as_ref();
        let rc_path = package.join(BoxUnboxRc::__rc_file_name());

        // TODO: do something if it already exists
        // WARN: this currently just overwrites the file if it already exists, BE CAREFUL!!
        let self_str = ron::ser::to_string_pretty(
            self,
            PrettyConfig::new()
                .struct_names(true)
                .separate_tuple_members(true),
        )?;
        fs::write(&rc_path, self_str)
            .with_context(|| format!("failed to write file: {rc_path:?}"))?;

        Ok(())
    }
}
