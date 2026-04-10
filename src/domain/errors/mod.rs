use thiserror::Error;

/// Domain-specific errors
#[derive(Debug, Error)]
pub enum DomainError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic domain error
    #[error("Domain error: {0}")]
    DomainError(String),

    /// Account not found
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Account disabled
    #[error("Account disabled: {0}")]
    AccountDisabled(String),

    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// Provider disabled
    #[error("Provider disabled: {0}")]
    ProviderDisabled(String),

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Access denied
    #[error("Access denied")]
    AccessDenied,

    /// Expired credentials
    #[error("Expired credentials")]
    ExpiredCredentials,

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Refresh token revoked
    #[error("Refresh token revoked. Re-authentication required.")]
    RefreshTokenRevoked,

    /// Invalid callback
    #[error("Invalid callback")]
    InvalidCallback,

    /// State mismatch (possible CSRF)
    #[error("State mismatch (possible CSRF)")]
    StateMismatch,

    /// Callback timeout
    #[error("Callback timeout")]
    CallbackTimeout,

    /// External service error
    #[error("External service error: {0}")]
    ExternalServiceError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error — source preserved via `#[from]`
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Lock acquisition timed out
    #[error("Lock timeout: {0}")]
    LockTimeout(String),

    /// Request exceeds model context window
    #[error("Token limit exceeded: {tokens} tokens exceed {limit} token context window for model '{model}'")]
    TokenLimitExceeded {
        model: String,
        tokens: u32,
        limit: u32,
    },
}
