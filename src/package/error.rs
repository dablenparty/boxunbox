use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ConfigRead {
    #[error("config file does not exist: {0}")]
    FileNotFound(PathBuf),
    #[error("failed to read '{path}': {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to parse TOML config: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("failed to parse RON config: {0}")]
    Ron(#[from] ron::error::SpannedError),
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
