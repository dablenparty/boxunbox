use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("no package config found for package `{0}`")]
    ConfigNotFound(PathBuf),
    #[error("failed to read package config: {0}")]
    FailedToReadFile(#[from] anyhow::Error),
    #[error("failed to parse package config as RON: {0}")]
    BadFormat(#[from] ron::error::SpannedError),
}

#[derive(Debug, Error)]
pub enum UnboxError {
    #[error("{0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("package doesn't exist: {0}")]
    PackageNotFound(PathBuf),
    #[error("failed to diff paths:\n  path: {path}\n  base: {base}")]
    PathDiffError { path: PathBuf, base: PathBuf },
    #[error("there's nothing to unbox!")]
    NothingToUnbox,
    #[error("missing write permissions for '{0}'")]
    NoWritePermissions(PathBuf),
    #[error("{0}")]
    ConfigParseError(#[from] ParseError),
    #[error("{0} already exists")]
    TargetAlreadyExists(PathBuf),
    #[error("failed to iterate package contents: {0}")]
    WalkdirError(#[from] walkdir::Error),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("failed to serialize into RON: `{0}`")]
    FailedToSerialize(#[from] ron::Error),
    #[error("failed to write RON to file: {0}")]
    FailedToWriteFile(#[from] io::Error),
}
