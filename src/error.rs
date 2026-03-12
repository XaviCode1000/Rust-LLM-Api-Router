//! Error handling with thiserror and anyhow
//!
//! This module provides the application-level error type using `thiserror`.
//! Domain errors are automatically converted to application errors.

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Environment error: {0}")]
    Env(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Metrics error: {0}")]
    Metrics(#[from] prometheus::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] crate::domain::errors::DomainError),
}

impl From<envy::Error> for Error {
    fn from(e: envy::Error) -> Self {
        Error::Env(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Internal(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Internal(e.to_string())
    }
}
