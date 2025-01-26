use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};

use crate::expand_into_pathbuf;

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

#[derive(Debug, thiserror::Error)]
pub enum RcParseError {
    #[error("no RC file found at `{0}`")]
    RcFileNotFound(PathBuf),
    #[error("failed to read rc file: {0}")]
    RcFileFailedToRead(#[from] anyhow::Error),
    #[error("failed to parse rc file: {0}")]
    BadRcFormat(#[from] ron::error::SpannedError),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BoxUnboxRc {
    #[serde(deserialize_with = "__de_pathbuf")]
    target: PathBuf,
}

impl BoxUnboxRc {
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
        // TODO: consider allowing multiple names
        const RC_FILE_NAME: &str = ".unboxrc.ron";

        let package = p.as_ref();
        let rc_file = package.join(RC_FILE_NAME);

        if !rc_file.exists() {
            return Err(RcParseError::RcFileNotFound(package.to_owned()));
        }

        let rc_str = fs::read_to_string(package)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let rc = ron::from_str(&rc_str)?;

        Ok(rc)
    }
}
