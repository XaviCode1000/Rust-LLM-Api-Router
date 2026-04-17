//! Shared test error type for failover integration tests.
//! Implements From<DomainError> so it can be used as the error type
//! in execute_with_failover generic calls.

use rust_llm_api_router::domain::DomainError;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct TestError(String);

/// Convenience constructors for common HTTP error types used in failover tests.
#[allow(dead_code)]
impl TestError {
    /// Creates a TestError representing a 429 Rate Limit error.
    #[inline]
    pub fn rate_limited() -> Self {
        TestError("Rate limit exceeded (429)".to_string())
    }

    /// Creates a TestError representing a 502 Bad Gateway error.
    #[inline]
    pub fn bad_gateway() -> Self {
        TestError("Bad Gateway (502)".to_string())
    }

    /// Creates a TestError representing a 503 Service Unavailable error.
    #[inline]
    pub fn service_unavailable() -> Self {
        TestError("Service Unavailable (503)".to_string())
    }

    /// Creates a TestError representing a 504 Gateway Timeout error.
    #[inline]
    pub fn gateway_timeout() -> Self {
        TestError("Gateway Timeout (504)".to_string())
    }

    /// Creates a generic TestError.
    #[inline]
    pub fn new(msg: &str) -> Self {
        TestError(msg.to_string())
    }

    /// Returns true if this is a 429 rate limit error.
    #[inline]
    pub fn is_rate_limit(&self) -> bool {
        self.0.contains("429") || self.0.contains("rate")
    }

    /// Returns true if this is a 5xx server error.
    #[inline]
    pub fn is_server_error(&self) -> bool {
        self.0.contains("502") || self.0.contains("503") || self.0.contains("504")
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl From<DomainError> for TestError {
    fn from(e: DomainError) -> Self {
        TestError(format!("DomainError: {}", e))
    }
}

/// Macro to create error variants for cleaner test code
#[macro_export]
#[allow(unused_macros)]
macro_rules! test_error {
    (rate_limit) => {
        $crate::common::errors::TestError::rate_limited()
    };
    (bad_gateway) => {
        $crate::common::errors::TestError::bad_gateway()
    };
    (unavailable) => {
        $crate::common::errors::TestError::service_unavailable()
    };
    (timeout) => {
        $crate::common::errors::TestError::gateway_timeout()
    };
    ($msg:expr) => {
        $crate::common::errors::TestError::new($msg)
    };
}
