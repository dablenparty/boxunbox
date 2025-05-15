use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read '{path}': {source}")]
    IoError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to deserialize TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}
