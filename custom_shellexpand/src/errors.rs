#[derive(Debug, thiserror::Error)]
pub enum ExpandError {
    #[error("matched envvar regex but failed to capture envvar")]
    EmptyEnvvarCapture,
    #[error("failed to get value of envvar: {0}")]
    EnvvarReadError(String),
}
