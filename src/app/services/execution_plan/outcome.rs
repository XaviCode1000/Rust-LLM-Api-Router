//! Execution Outcome
//!
//! Defines the possible outcomes of an execution plan.

use serde::{Deserialize, Serialize};

/// Outcome of an execution plan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionOutcome {
    /// Execution succeeded with primary account
    Success,

    /// Execution failed with all accounts/providers
    Failure,

    /// Execution succeeded with fallback account/provider
    Fallback,
}

impl ExecutionOutcome {
    /// Returns true if the execution was successful (including fallback).
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::Fallback)
    }

    /// Returns true if fallback was used.
    pub fn used_fallback(&self) -> bool {
        matches!(self, Self::Fallback)
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Success => "Execution succeeded with primary account",
            Self::Failure => "Execution failed with all accounts/providers",
            Self::Fallback => "Execution succeeded with fallback account",
        }
    }
}

impl std::fmt::Display for ExecutionOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Fallback => write!(f, "fallback"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_is_success() {
        assert!(ExecutionOutcome::Success.is_success());
        assert!(ExecutionOutcome::Fallback.is_success());
        assert!(!ExecutionOutcome::Failure.is_success());
    }

    #[test]
    fn test_outcome_used_fallback() {
        assert!(!ExecutionOutcome::Success.used_fallback());
        assert!(ExecutionOutcome::Fallback.used_fallback());
        assert!(!ExecutionOutcome::Failure.used_fallback());
    }

    #[test]
    fn test_outcome_display() {
        assert_eq!(ExecutionOutcome::Success.to_string(), "success");
        assert_eq!(ExecutionOutcome::Failure.to_string(), "failure");
        assert_eq!(ExecutionOutcome::Fallback.to_string(), "fallback");
    }
}
