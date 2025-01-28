use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("no package config found at `{0}`")]
    FileNotFound(PathBuf),
    #[error("failed to read package config: {0}")]
    FailedToReadFile(#[from] anyhow::Error),
    #[error("failed to parse package config as RON: {0}")]
    BadFormat(#[from] ron::error::SpannedError),
}

#[derive(Debug, Error)]
pub enum UnboxError {
    #[error("package doesn't exist: {0}")]
    PackageNotFound(PathBuf),
    #[error("{0}")]
    AnyhowError(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("failed to serialize into RON: `{0}`")]
    FailedToSerialize(#[from] ron::Error),
    #[error("failed to write RON to file: {0}")]
    FailedToWriteFile(#[from] io::Error),
}
