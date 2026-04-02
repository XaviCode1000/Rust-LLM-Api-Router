//! Shared test error type for failover integration tests.
//! Implements From<DomainError> so it can be used as the error type
//! in execute_with_failover generic calls.

use rust_llm_api_router::domain::DomainError;
use std::fmt;

#[derive(Clone, Debug)]
pub struct TestError(String);

impl TestError {
    #[allow(dead_code)]
    pub fn new(msg: &str) -> Self {
        TestError(msg.to_string())
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<DomainError> for TestError {
    fn from(e: DomainError) -> Self {
        TestError(format!("DomainError: {}", e))
    }
}
