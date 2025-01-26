use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
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

/// An error that can occur when writing/saving the RC file.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// Failed to serialize struct
    #[error("failed to serialize RC struct: {0}")]
    RcFailedToSerialize(#[from] ron::Error),
    /// Failed to write to file
    #[error("failed to write rc file: {0}")]
    RcFailedToWrite(#[from] anyhow::Error),
}
