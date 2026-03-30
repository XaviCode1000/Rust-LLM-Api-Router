//! Execution Context
//!
//! Provides types for capturing request metadata and planning options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for planning an LLM request execution.
///
/// This struct captures all the metadata needed to select an appropriate
/// execution strategy and route the request to the right account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier for this request
    pub request_id: String,

    /// Model being requested (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// Preferred provider IDs (empty = any available provider)
    pub preferred_providers: Vec<String>,

    /// Preferred account IDs (empty = any available account)
    pub preferred_accounts: Vec<String>,

    /// Provider/Model preferences (e.g., "openai:gpt-4", "anthropic:claude-3")
    pub provider_model_preferences: Vec<String>,

    /// Request metadata (temperature, max_tokens, etc.)
    pub request_params: HashMap<String, serde_json::Value>,

    /// Planning options for this request
    pub planning_options: PlanningOptions,
}

impl ExecutionContext {
    /// Creates a new `ExecutionContext` with required fields.
    pub fn new(request_id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            model: model.into(),
            preferred_providers: Vec::new(),
            preferred_accounts: Vec::new(),
            provider_model_preferences: Vec::new(),
            request_params: HashMap::new(),
            planning_options: PlanningOptions::default(),
        }
    }

    /// Sets preferred providers for this request.
    pub fn with_preferred_providers(mut self, providers: Vec<String>) -> Self {
        self.preferred_providers = providers;
        self
    }

    /// Sets preferred accounts for this request.
    pub fn with_preferred_accounts(mut self, accounts: Vec<String>) -> Self {
        self.preferred_accounts = accounts;
        self
    }

    /// Sets provider-model preferences.
    pub fn with_provider_model_preferences(mut self, preferences: Vec<String>) -> Self {
        self.provider_model_preferences = preferences;
        self
    }

    /// Adds a request parameter.
    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.request_params.insert(key.into(), value);
        self
    }

    /// Sets planning options.
    pub fn with_planning_options(mut self, options: PlanningOptions) -> Self {
        self.planning_options = options;
        self
    }

    /// Checks if a specific provider is preferred.
    pub fn is_provider_preferred(&self, provider_id: &str) -> bool {
        self.preferred_providers.is_empty()
            || self.preferred_providers.iter().any(|p| p == provider_id)
    }

    /// Checks if a specific account is preferred.
    pub fn is_account_preferred(&self, account_id: &str) -> bool {
        self.preferred_accounts.is_empty()
            || self.preferred_accounts.iter().any(|a| a == account_id)
    }
}

/// Options for planning an execution strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanningOptions {
    /// Enable failover on provider failure
    pub enable_failover: bool,

    /// Enable load balancing across accounts
    pub enable_load_balancing: bool,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Timeout for the entire request in seconds
    pub timeout_seconds: u32,

    /// Enable cost optimization (prefer cheaper providers)
    pub cost_optimized: bool,

    /// Enable health-aware routing (prefer healthy accounts)
    pub health_aware_routing: bool,

    /// Enable cascading execution (quality-based escalation)
    pub enable_cascading: bool,

    /// Enable budget optimization (consider cost-effective execution strategies)
    pub budget_mode: bool,

    /// Custom priority for this request
    pub priority: i32,
}

impl Default for PlanningOptions {
    fn default() -> Self {
        Self {
            enable_failover: true,
            enable_load_balancing: true,
            max_retries: 3,
            timeout_seconds: 60,
            cost_optimized: false,
            health_aware_routing: true,
            enable_cascading: false,
            budget_mode: false,
            priority: 0,
        }
    }
}

impl PlanningOptions {
    /// Creates a new `PlanningOptions` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates options optimized for reliability.
    pub fn reliability() -> Self {
        Self {
            enable_failover: true,
            enable_load_balancing: true,
            max_retries: 5,
            timeout_seconds: 120,
            cost_optimized: false,
            health_aware_routing: true,
            enable_cascading: false,
            budget_mode: false,
            priority: 10,
        }
    }

    /// Creates options optimized for low cost.
    pub fn cost_optimized() -> Self {
        Self {
            enable_failover: false,
            enable_load_balancing: false,
            max_retries: 1,
            timeout_seconds: 30,
            cost_optimized: true,
            health_aware_routing: false,
            enable_cascading: false,
            budget_mode: true,
            priority: -10,
        }
    }

    /// Creates options optimized for low latency.
    pub fn low_latency() -> Self {
        Self {
            enable_failover: false,
            enable_load_balancing: true,
            max_retries: 1,
            timeout_seconds: 15,
            cost_optimized: false,
            health_aware_routing: true,
            enable_cascading: false,
            budget_mode: false,
            priority: 5,
        }
    }

    /// Creates options optimized for cascading execution.
    pub fn cascading() -> Self {
        Self {
            enable_failover: false,
            enable_load_balancing: false,
            max_retries: 1,
            timeout_seconds: 30,
            cost_optimized: true,
            health_aware_routing: true,
            enable_cascading: true,
            budget_mode: true,
            priority: 0,
        }
    }
}

impl PlanningOptions {
    /// Sets enable_failover.
    pub fn with_failover(mut self, enabled: bool) -> Self {
        self.enable_failover = enabled;
        self
    }

    /// Sets enable_load_balancing.
    pub fn with_load_balancing(mut self, enabled: bool) -> Self {
        self.enable_load_balancing = enabled;
        self
    }

    /// Sets max_retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Sets timeout_seconds.
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_new() {
        let ctx = ExecutionContext::new("req-1", "gpt-4");
        assert_eq!(ctx.request_id, "req-1");
        assert_eq!(ctx.model, "gpt-4");
        assert!(ctx.preferred_providers.is_empty());
        assert!(ctx.planning_options.enable_failover);
    }

    #[test]
    fn test_execution_context_preferred_providers() {
        // Test with specific provider preference
        let ctx = ExecutionContext::new("req-1", "gpt-4")
            .with_preferred_providers(vec!["openai".to_string()]);

        assert!(ctx.is_provider_preferred("openai"));
        assert!(!ctx.is_provider_preferred("anthropic")); // Only openai is preferred

        // Test with empty preference (any provider)
        let ctx_any = ExecutionContext::new("req-2", "gpt-4");
        assert!(ctx_any.is_provider_preferred("any"));
        assert!(ctx_any.is_provider_preferred("openai"));
    }

    #[test]
    fn test_planning_options_defaults() {
        let opts = PlanningOptions::default();
        assert!(opts.enable_failover);
        assert!(opts.enable_load_balancing);
        assert_eq!(opts.max_retries, 3);
    }

    #[test]
    fn test_planning_options_presets() {
        let reliability = PlanningOptions::reliability();
        assert!(reliability.enable_failover);
        assert_eq!(reliability.max_retries, 5);

        let cost = PlanningOptions::cost_optimized();
        assert!(cost.cost_optimized);
        assert_eq!(cost.max_retries, 1);

        let latency = PlanningOptions::low_latency();
        assert!(latency.health_aware_routing);
        assert_eq!(latency.timeout_seconds, 15);
    }

    #[test]
    fn test_planning_options_cascading() {
        let cascading = PlanningOptions::cascading();
        assert!(!cascading.enable_failover);
        assert!(!cascading.enable_load_balancing);
        assert_eq!(cascading.max_retries, 1);
        assert_eq!(cascading.timeout_seconds, 30);
        assert!(cascading.cost_optimized);
        assert!(cascading.health_aware_routing);
        assert!(cascading.enable_cascading);
        assert_eq!(cascading.priority, 0);
    }
}
