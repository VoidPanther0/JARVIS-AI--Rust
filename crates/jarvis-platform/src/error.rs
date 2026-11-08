use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("action denied by user")]
    Denied,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PlatformResult<T> = Result<T, PlatformError>;
