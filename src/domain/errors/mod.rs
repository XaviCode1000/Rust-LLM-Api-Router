//! Domain errors
//!
//! Custom error types for the domain layer using thiserror.
//! These errors represent business logic failures and validation errors.

use thiserror::Error;

/// Domain-level errors for the LLM API Router.
///
/// # Variants
/// * `InvalidRequest` - The request failed validation
/// * `ProviderNotFound` - The requested provider does not exist
/// * `ProviderDisabled` - The provider exists but is disabled
/// * `AccountNotFound` - The requested account does not exist
/// * `AccountInactive` - The account exists but is inactive
/// * `NoAvailableAccounts` - No active accounts available for the provider
/// * `ModelNotFound` - The requested model does not exist
/// * `GatewayError` - Error communicating with the LLM provider
/// * `AuthenticationError` - Invalid or missing API key
/// * `RateLimited` - Too many requests to the provider
/// * `ValidationError` - Data validation failed
/// * `Io` - I/O operation failed
/// * `Serialization` - Serialization/deserialization failed
/// * `Internal` - Internal domain error
#[derive(Debug, Error)]
pub enum DomainError {
    /// The request failed validation
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// The requested provider does not exist
    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    /// The provider exists but is disabled
    #[error("provider disabled: {0}")]
    ProviderDisabled(String),

    /// The requested account does not exist
    #[error("account not found: {0}")]
    AccountNotFound(String),

    /// The account exists but is inactive
    #[error("account inactive: {0}")]
    AccountInactive(String),

    /// No active accounts available for the provider
    #[error("no available accounts for provider: {0}")]
    NoAvailableAccounts(String),

    /// The requested model does not exist
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Error communicating with the LLM provider
    #[error("gateway error: {0}")]
    GatewayError(String),

    /// Invalid or missing API key
    #[error("authentication error: {0}")]
    AuthenticationError(String),

    /// Too many requests to the provider
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// Data validation failed
    #[error("validation error: {0}")]
    ValidationError(String),

    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(String),

    /// Serialization/deserialization failed
    #[error("serialization error: {0}")]
    Serialization(String),

    /// External service error (provider API failure)
    #[error("external service error: {0}")]
    ExternalServiceError(String),

    /// Feature not implemented
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// Internal domain error
    #[error("internal error: {0}")]
    Internal(String),
}

impl DomainError {
    /// Creates an `InvalidRequest` error.
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    /// Creates a `ProviderNotFound` error.
    pub fn provider_not_found(id: impl Into<String>) -> Self {
        Self::ProviderNotFound(id.into())
    }

    /// Creates a `ProviderDisabled` error.
    pub fn provider_disabled(id: impl Into<String>) -> Self {
        Self::ProviderDisabled(id.into())
    }

    /// Creates an `AccountNotFound` error.
    pub fn account_not_found(id: impl Into<String>) -> Self {
        Self::AccountNotFound(id.into())
    }

    /// Creates an `AccountInactive` error.
    pub fn account_inactive(id: impl Into<String>) -> Self {
        Self::AccountInactive(id.into())
    }

    /// Creates a `NoAvailableAccounts` error.
    pub fn no_available_accounts(provider_id: impl Into<String>) -> Self {
        Self::NoAvailableAccounts(provider_id.into())
    }

    /// Creates a `ModelNotFound` error.
    pub fn model_not_found(id: impl Into<String>) -> Self {
        Self::ModelNotFound(id.into())
    }

    /// Creates a `GatewayError` error.
    pub fn gateway_error(msg: impl Into<String>) -> Self {
        Self::GatewayError(msg.into())
    }

    /// Creates an `AuthenticationError` error.
    pub fn authentication_error(msg: impl Into<String>) -> Self {
        Self::AuthenticationError(msg.into())
    }

    /// Creates a `RateLimited` error.
    pub fn rate_limited(msg: impl Into<String>) -> Self {
        Self::RateLimited(msg.into())
    }

    /// Creates a `ValidationError` error.
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Creates an `Io` error.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Creates a `Serialization` error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Creates an `ExternalServiceError` error.
    pub fn external_service_error(msg: impl Into<String>) -> Self {
        Self::ExternalServiceError(msg.into())
    }

    /// Creates a `NotImplemented` error.
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Self::NotImplemented(msg.into())
    }

    /// Creates an `Internal` error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Result type alias for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod mod_tests;
