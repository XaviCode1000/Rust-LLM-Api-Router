//! Execution Plan Types
//!
//! Defines different execution plan strategies and account planning structures.

use serde::{Deserialize, Serialize};

use crate::domain::entities::{AccountHealth, Provider};

/// Type of execution plan strategy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ExecutionPlanType {
    /// Standard execution with single account
    #[default]
    Standard,

    /// Execution with automatic failover
    Failover,

    /// Load-balanced execution across accounts
    LoadBalanced,

    /// Cost-optimized execution
    CostOptimized,

    /// Cascading execution: try cheapest first, escalate on quality failure
    Cascading,
}

impl ExecutionPlanType {
    /// Returns a human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Failover => "Failover",
            Self::LoadBalanced => "Load Balanced",
            Self::CostOptimized => "Cost Optimized",
            Self::Cascading => "Cascading",
        }
    }

    /// Returns true if this plan type supports failover.
    pub fn supports_failover(&self) -> bool {
        matches!(self, Self::Failover | Self::LoadBalanced)
    }

    /// Returns true if this plan type supports load balancing.
    pub fn supports_load_balancing(&self) -> bool {
        matches!(self, Self::LoadBalanced)
    }

    /// Returns true if this plan type is cost-optimized.
    pub fn is_cost_optimized(&self) -> bool {
        matches!(self, Self::CostOptimized | Self::Cascading)
    }

    /// Returns true if this plan type supports quality-based escalation.
    pub fn supports_cascading(&self) -> bool {
        matches!(self, Self::Cascading)
    }
}

impl std::fmt::Display for ExecutionPlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A planned account with provider info and health snapshot.
///
/// This represents a single account in an execution plan with all the
/// metadata needed to execute a request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlannedAccount {
    /// Account identifier
    pub account_id: String,

    /// Provider identifier
    pub provider_id: String,

    /// Provider name for display
    pub provider_name: String,

    /// Provider base URL
    pub provider_base_url: String,

    /// Health snapshot at planning time
    pub health_snapshot: AccountHealth,

    /// Priority of this account (lower = higher priority)
    pub priority: i32,

    /// Whether this is the primary account
    pub is_primary: bool,

    /// Whether this is a fallback account
    pub is_fallback: bool,

    /// Order in the execution sequence
    pub execution_order: u32,

    /// Model ID associated with this account (used in cascading plans)
    pub model_id: Option<String>,
}

impl PlannedAccount {
    /// Creates a new `PlannedAccount` from an account and provider.
    pub fn new(account_id: impl Into<String>, provider: &Provider, health: AccountHealth) -> Self {
        Self {
            account_id: account_id.into(),
            provider_id: provider.id.to_string(),
            provider_name: provider.name.clone(),
            provider_base_url: provider.base_url.clone(),
            health_snapshot: health,
            priority: 0,
            is_primary: true,
            is_fallback: false,
            execution_order: 0,
            model_id: None,
        }
    }

    /// Sets the account as primary.
    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self.is_fallback = false;
        self
    }

    /// Sets the account as fallback.
    pub fn as_fallback(mut self) -> Self {
        self.is_primary = false;
        self.is_fallback = true;
        self
    }

    /// Sets the priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the execution order.
    pub fn with_execution_order(mut self, order: u32) -> Self {
        self.execution_order = order;
        self
    }

    /// Sets the model ID for this account.
    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    /// Returns the health score (0-100).
    pub fn health_score(&self) -> f64 {
        self.health_snapshot.health_score()
    }

    /// Returns true if the account is healthy enough to use.
    pub fn is_healthy(&self) -> bool {
        !self.health_snapshot.circuit_breaker_open() && self.health_score() > 0.0
    }

    /// Returns the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        self.health_snapshot.success_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan_type_name() {
        assert_eq!(ExecutionPlanType::Standard.name(), "Standard");
        assert_eq!(ExecutionPlanType::Failover.name(), "Failover");
        assert_eq!(ExecutionPlanType::LoadBalanced.name(), "Load Balanced");
        assert_eq!(ExecutionPlanType::CostOptimized.name(), "Cost Optimized");
    }

    #[test]
    fn test_execution_plan_type_supports() {
        assert!(!ExecutionPlanType::Standard.supports_failover());
        assert!(ExecutionPlanType::Failover.supports_failover());
        assert!(ExecutionPlanType::LoadBalanced.supports_failover());

        assert!(!ExecutionPlanType::Standard.supports_load_balancing());
        assert!(ExecutionPlanType::LoadBalanced.supports_load_balancing());

        assert!(!ExecutionPlanType::Standard.is_cost_optimized());
        assert!(ExecutionPlanType::CostOptimized.is_cost_optimized());
    }

    #[test]
    fn test_planned_account() {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health = AccountHealth::new("acc-1");
        let planned = PlannedAccount::new("acc-1", &provider, health);

        assert_eq!(planned.account_id, "acc-1");
        assert_eq!(planned.provider_id, "openai");
        assert!(planned.is_primary);
        assert!(!planned.is_fallback);
    }

    #[test]
    fn test_planned_account_fallback() {
        let provider = Provider::new("anthropic", "Anthropic", "https://api.anthropic.com");
        let health = AccountHealth::new("acc-2");
        let planned = PlannedAccount::new("acc-2", &provider, health)
            .as_fallback()
            .with_execution_order(1);

        assert!(!planned.is_primary);
        assert!(planned.is_fallback);
        assert_eq!(planned.execution_order, 1);
    }

    #[test]
    fn test_planned_account_health() {
        let provider = Provider::new("test", "Test", "https://test.com");
        let mut health = AccountHealth::new("acc-1");

        // Record some successes
        health.record_success(100);
        health.record_success(150);
        health.record_success(200);

        let planned = PlannedAccount::new("acc-1", &provider, health);

        assert!(planned.is_healthy());
        assert!(planned.success_rate() > 0.0);
    }
}
