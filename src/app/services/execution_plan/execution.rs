//! Execution Execution Context
//!
//! Provides types for executing execution plans.

use serde::{Deserialize, Serialize};

/// Configuration for executing an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Enable streaming mode (if true, cascading is skipped)
    pub stream: bool,
    /// Cost budget in microdollars (0 = no budget limit)
    pub max_cost_microdollars: u64,
    /// Whether to perform quality-based escalation
    pub enable_quality_escalation: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            stream: false,
            max_cost_microdollars: 0,
            enable_quality_escalation: true,
        }
    }
}

impl ExecutionConfig {
    /// Creates a non-streaming execution config.
    pub fn non_streaming() -> Self {
        Self {
            stream: false,
            max_cost_microdollars: 0,
            enable_quality_escalation: true,
        }
    }

    /// Creates a streaming execution config (no cascading).
    pub fn streaming() -> Self {
        Self {
            stream: true,
            max_cost_microdollars: 0,
            enable_quality_escalation: false,
        }
    }

    /// Creates a config with a cost budget.
    pub fn with_cost_budget(mut self, max_cost_microdollars: u64) -> Self {
        self.max_cost_microdollars = max_cost_microdollars;
        self
    }
}

/// Result of executing an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Final tier that succeeded (index)
    pub final_tier_index: usize,
    /// Response text (if successful)
    pub response: Option<String>,
    /// Tokens used (input + output)
    pub tokens_used: Option<(u64, u64)>,
    /// Cost in microdollars for the final tier
    pub final_cost_microdollars: u64,
    /// Total cost accumulated across all attempts
    pub total_cost_microdollars: u64,
    /// Quality score of the final response (if evaluated)
    pub final_quality_score: Option<f64>,
    /// Whether quality escalation was needed
    pub used_quality_escalation: bool,
}

impl ExecutionResult {
    /// Creates a failed execution result.
    pub fn failure() -> Self {
        Self {
            success: false,
            final_tier_index: 0,
            response: None,
            tokens_used: None,
            final_cost_microdollars: 0,
            total_cost_microdollars: 0,
            final_quality_score: None,
            used_quality_escalation: false,
        }
    }

    /// Creates a successful execution result.
    pub fn success(
        final_tier_index: usize,
        response: String,
        tokens_used: (u64, u64),
        final_cost_microdollars: u64,
        total_cost_microdollars: u64,
        final_quality_score: Option<f64>,
        used_quality_escalation: bool,
    ) -> Self {
        Self {
            success: true,
            final_tier_index,
            response: Some(response),
            tokens_used: Some(tokens_used),
            final_cost_microdollars,
            total_cost_microdollars,
            final_quality_score,
            used_quality_escalation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert!(!config.stream);
        assert!(config.enable_quality_escalation);
        assert_eq!(config.max_cost_microdollars, 0);
    }

    #[test]
    fn test_execution_config_non_streaming() {
        let config = ExecutionConfig::non_streaming();
        assert!(!config.stream);
    }

    #[test]
    fn test_execution_config_streaming() {
        let config = ExecutionConfig::streaming();
        assert!(config.stream);
        assert!(!config.enable_quality_escalation);
    }

    #[test]
    fn test_execution_config_with_budget() {
        let config = ExecutionConfig::default().with_cost_budget(5000);
        assert_eq!(config.max_cost_microdollars, 5000);
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::failure();
        assert!(!result.success);
        assert_eq!(result.total_cost_microdollars, 0);
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success(
            1,
            "Test response".to_string(),
            (100, 50),
            2000,
            3000,
            Some(0.9),
            true,
        );
        assert!(result.success);
        assert_eq!(result.final_tier_index, 1);
        assert_eq!(result.total_cost_microdollars, 3000);
        assert_eq!(result.final_quality_score, Some(0.9));
        assert!(result.used_quality_escalation);
    }
}
