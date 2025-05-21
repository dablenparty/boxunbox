use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ReadError {
    #[error("failed to read '{path}': {source}")]
    IoError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to deserialize TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Debug, ThisError)]
pub enum WriteError {
    #[error("failed to write to '{path}': {source}")]
    IoError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to serialize to TOML: {0}")]
    TomlError(#[from] toml::ser::Error),
}
