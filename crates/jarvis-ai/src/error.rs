use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("no AI provider named '{0}' is registered")]
    ProviderNotFound(String),

    #[error("provider '{provider}' failed: {message}")]
    ProviderFailure { provider: String, message: String },

    #[error("no API key is configured for provider '{0}'")]
    NoKeyConfigured(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("request to '{provider}' timed out after {timeout_secs}s")]
    Timeout { provider: String, timeout_secs: u64 },

    #[error("request to '{provider}' was cancelled")]
    Cancelled { provider: String },

    #[error("'{provider}' rejected the API key (HTTP {status})")]
    Unauthorized { provider: String, status: u16 },

    #[error("'{provider}' rate-limited this request (HTTP 429)")]
    RateLimited { provider: String },

    #[error("'{provider}' returned a server error (HTTP {status}): {message}")]
    ServerError {
        provider: String,
        status: u16,
        message: String,
    },

    #[error(
        "no compatible free model is currently available for provider '{provider}' \
         (task: {task}); {attempted} candidate(s) were tried and all failed"
    )]
    NoModelAvailable {
        provider: String,
        task: String,
        attempted: usize,
    },
}

pub type AiResult<T> = Result<T, AiError>;
