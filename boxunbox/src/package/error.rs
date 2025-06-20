use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ConfigRead {
    #[error("config file does not exist: {0}")]
    FileNotFound(PathBuf),
    #[error("failed to read '{path}'")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to parse TOML config")]
    Toml(#[from] toml::de::Error),
    #[error("failed to parse RON config")]
    Ron(#[from] ron::error::SpannedError),
}

#[derive(Debug, ThisError)]
pub enum ConfigWrite {
    #[error("failed to write to '{path}'")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to serialize to TOML")]
    Toml(#[from] toml::ser::Error),
}
