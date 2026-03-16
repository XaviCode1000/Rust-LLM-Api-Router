//! Execution Plan Status
//!
//! Defines the possible states of an execution plan.

use serde::{Deserialize, Serialize};

/// Status of an execution plan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionPlanStatus {
    /// Plan has been created but not yet executed
    Planned,

    /// Plan is currently being executed
    InProgress,

    /// Plan execution completed successfully
    Completed,

    /// Plan execution failed
    Failed,
}

impl ExecutionPlanStatus {
    /// Returns true if the plan is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Returns true if the plan can be retried.
    pub fn is_retriable(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Planned => "Plan created, awaiting execution",
            Self::InProgress => "Currently executing",
            Self::Completed => "Execution completed successfully",
            Self::Failed => "Execution failed",
        }
    }
}

impl std::fmt::Display for ExecutionPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl Default for ExecutionPlanStatus {
    fn default() -> Self {
        Self::Planned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_terminal() {
        assert!(!ExecutionPlanStatus::Planned.is_terminal());
        assert!(!ExecutionPlanStatus::InProgress.is_terminal());
        assert!(ExecutionPlanStatus::Completed.is_terminal());
        assert!(ExecutionPlanStatus::Failed.is_terminal());
    }

    #[test]
    fn test_status_retriable() {
        assert!(!ExecutionPlanStatus::Planned.is_retriable());
        assert!(!ExecutionPlanStatus::InProgress.is_retriable());
        assert!(!ExecutionPlanStatus::Completed.is_retriable());
        assert!(ExecutionPlanStatus::Failed.is_retriable());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(ExecutionPlanStatus::Planned.to_string(), "planned");
        assert_eq!(ExecutionPlanStatus::InProgress.to_string(), "in_progress");
        assert_eq!(ExecutionPlanStatus::Completed.to_string(), "completed");
        assert_eq!(ExecutionPlanStatus::Failed.to_string(), "failed");
    }
}
