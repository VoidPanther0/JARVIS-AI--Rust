use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("no AI provider named '{0}' is registered")]
    ProviderNotFound(String),

    #[error("provider '{provider}' failed: {message}")]
    ProviderFailure { provider: String, message: String },
}

pub type AiResult<T> = Result<T, AiError>;
