use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ConfigRead {
    #[error("failed to read '{path}': {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to deserialize TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, ThisError)]
pub enum ConfigWrite {
    #[error("failed to write to '{path}': {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to serialize to TOML: {0}")]
    Toml(#[from] toml::ser::Error),
}
