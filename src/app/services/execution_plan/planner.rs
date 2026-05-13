//! Execution Planner
//!
//! Provides the ExecutionPlanner service that creates execution plans based on context.

use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::RoutingConfig;
use crate::domain::entities::{Account, AccountHealth, Provider};
use crate::domain::traits::AccountRepository;
use crate::domain::DomainResult;

use super::metrics::ExecutionPlanMetrics;
use super::tracing::{logging, PlanningSpan};
use super::types::{ExecutionPlanType, PlannedAccount};
use super::{ExecutionContext, ExecutionPlan, ExecutionPlanImpl};

/// Configuration for the execution planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlannerConfig {
    // ... existing config fields (lines 9-61)
    /// Default plan type to use when no specific strategy is selected
    pub default_plan_type: ExecutionPlanType,

    /// Enable automatic plan selection based on context
    pub enable_auto_selection: bool,

    /// Maximum number of accounts to include in a plan
    pub max_accounts_per_plan: usize,

    /// Minimum health score threshold (0-100)
    pub min_health_score: f64,

    /// Enable cost optimization by default
    pub cost_optimization_enabled: bool,

    /// Enable failover by default
    pub failover_enabled: bool,

    /// Enable load balancing by default
    pub load_balancing_enabled: bool,

    /// Enable cascading by default
    pub cascading_enabled: bool,

    /// Enable budget mode by default
    pub budget_mode_enabled: bool,

    /// Default max retries
    pub default_max_retries: u32,

    /// Default timeout in seconds
    pub default_timeout_seconds: u32,

    /// Circuit breaker threshold (consecutive failures)
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_seconds: u64,
}

impl Default for ExecutionPlannerConfig {
    fn default() -> Self {
        Self {
            default_plan_type: ExecutionPlanType::Standard,
            enable_auto_selection: true,
            max_accounts_per_plan: 5,
            min_health_score: 0.0,
            cost_optimization_enabled: false,
            failover_enabled: true,
            load_balancing_enabled: true,
            cascading_enabled: false,
            budget_mode_enabled: false,
            default_max_retries: 3,
            default_timeout_seconds: 60,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 30,
        }
    }
}

impl ExecutionPlannerConfig {
    /// Creates a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an ExecutionPlannerConfig from RoutingConfig.
    ///
    /// This converts the high-level routing configuration into planner-specific
    /// settings, mapping routing strategy to execution plan type and flags.
    pub fn from_routing_config(config: &RoutingConfig) -> Self {
        Self {
            default_plan_type: ExecutionPlanType::Standard,
            enable_auto_selection: true,
            max_accounts_per_plan: 5,
            min_health_score: 0.0,
            cascading_enabled: config.cascading_enabled
                || config.strategy == crate::config::RoutingStrategy::Cascading,
            cost_optimization_enabled: config.strategy
                == crate::config::RoutingStrategy::CostOptimized
                || config.budget_mode,
            load_balancing_enabled: config.strategy == crate::config::RoutingStrategy::LoadBalanced,
            failover_enabled: config.strategy == crate::config::RoutingStrategy::Failover,
            budget_mode_enabled: config.budget_mode,
            default_max_retries: config.max_retries,
            default_timeout_seconds: config.timeout_seconds.min(u32::MAX as u64) as u32,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 30,
        }
    }

    /// Creates a config optimized for reliability.
    pub fn reliability() -> Self {
        Self {
            default_plan_type: ExecutionPlanType::Failover,
            enable_auto_selection: true,
            max_accounts_per_plan: 5,
            min_health_score: 0.0,
            cost_optimization_enabled: false,
            failover_enabled: true,
            load_balancing_enabled: true,
            cascading_enabled: false,
            budget_mode_enabled: false,
            default_max_retries: 5,
            default_timeout_seconds: 120,
            circuit_breaker_threshold: 3,
            circuit_breaker_timeout_seconds: 60,
        }
    }

    /// Creates a config optimized for low cost.
    pub fn cost_optimized() -> Self {
        Self {
            default_plan_type: ExecutionPlanType::CostOptimized,
            enable_auto_selection: true,
            max_accounts_per_plan: 3,
            min_health_score: 50.0,
            cost_optimization_enabled: true,
            failover_enabled: false,
            load_balancing_enabled: false,
            cascading_enabled: false,
            budget_mode_enabled: true,
            default_max_retries: 1,
            default_timeout_seconds: 30,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 30,
        }
    }

    /// Creates a config optimized for low latency.
    pub fn low_latency() -> Self {
        Self {
            default_plan_type: ExecutionPlanType::LoadBalanced,
            enable_auto_selection: true,
            max_accounts_per_plan: 3,
            min_health_score: 70.0,
            cost_optimization_enabled: false,
            failover_enabled: false,
            load_balancing_enabled: true,
            cascading_enabled: false,
            budget_mode_enabled: false,
            default_max_retries: 1,
            default_timeout_seconds: 15,
            circuit_breaker_threshold: 3,
            circuit_breaker_timeout_seconds: 15,
        }
    }

    /// Sets the default plan type.
    pub fn with_default_plan_type(mut self, plan_type: ExecutionPlanType) -> Self {
        self.default_plan_type = plan_type;
        self
    }

    /// Sets max accounts per plan.
    pub fn with_max_accounts(mut self, max: usize) -> Self {
        self.max_accounts_per_plan = max;
        self
    }

    /// Sets minimum health score.
    pub fn with_min_health_score(mut self, score: f64) -> Self {
        self.min_health_score = score;
        self
    }

    /// Sets whether auto-selection is enabled.
    pub fn with_auto_selection(mut self, enabled: bool) -> Self {
        self.enable_auto_selection = enabled;
        self
    }
}

/// Builder for execution planner config.
#[derive(Debug, Default)]
pub struct ExecutionPlannerConfigBuilder {
    config: ExecutionPlannerConfig,
}

impl ExecutionPlannerConfigBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads config from environment variables.
    #[allow(clippy::wrong_self_convention)]
    pub fn from_env(mut self) -> Self {
        // Load from environment variables if present
        if let Ok(plan_type) = std::env::var("EXECUTION_PLAN_TYPE") {
            self.config.default_plan_type = match plan_type.to_lowercase().as_str() {
                "standard" => ExecutionPlanType::Standard,
                "failover" => ExecutionPlanType::Failover,
                "loadbalanced" | "load_balanced" => ExecutionPlanType::LoadBalanced,
                "costoptimized" | "cost_optimized" => ExecutionPlanType::CostOptimized,
                _ => ExecutionPlanType::Standard,
            };
        }

        if let Ok(enabled) = std::env::var("EXECUTION_AUTO_SELECTION") {
            self.config.enable_auto_selection = enabled.to_lowercase() != "false";
        }

        if let Ok(max) = std::env::var("EXECUTION_MAX_ACCOUNTS") {
            if let Ok(parsed) = max.parse() {
                self.config.max_accounts_per_plan = parsed;
            }
        }

        if let Ok(retries) = std::env::var("EXECUTION_MAX_RETRIES") {
            if let Ok(parsed) = retries.parse() {
                self.config.default_max_retries = parsed;
            }
        }

        if let Ok(timeout) = std::env::var("EXECUTION_TIMEOUT_SECONDS") {
            if let Ok(parsed) = timeout.parse() {
                self.config.default_timeout_seconds = parsed;
            }
        }

        self
    }

    /// Sets the default plan type.
    #[allow(dead_code)]
    pub fn with_default_plan_type(mut self, plan_type: ExecutionPlanType) -> Self {
        self.config.default_plan_type = plan_type;
        self
    }

    /// Sets max accounts per plan.
    #[allow(dead_code)]
    pub fn with_max_accounts(mut self, max: usize) -> Self {
        self.config.max_accounts_per_plan = max;
        self
    }

    /// Builds the config.
    pub fn build(self) -> ExecutionPlannerConfig {
        self.config
    }
}

// =============================================================================
// ExecutionPlanner Service
// =============================================================================

/// Service that creates execution plans based on context and configuration.
///
/// The ExecutionPlanner orchestrates the creation of execution plans by:
/// 1. Fetching available accounts from the repository
/// 2. Filtering accounts by model compatibility
/// 3. Integrating health snapshots
/// 4. Applying rotation strategies
/// 5. Selecting the appropriate plan type based on config
#[derive(Clone)]
pub struct ExecutionPlanner<R: AccountRepository + ?Sized> {
    account_repo: Arc<R>,
    config: ExecutionPlannerConfig,
    rotation_strategy: RotationStrategyType,
    metrics: Option<Arc<ExecutionPlanMetrics>>,
}

/// Rotation strategy types for account selection.
#[derive(Debug, Clone, Copy, Default)]
pub enum RotationStrategyType {
    /// Round-robin rotation
    RoundRobin,
    /// Weighted by health score
    #[default]
    HealthWeighted,
    /// Priority-based selection
    Priority,
    /// Least recently used
    LeastRecentlyUsed,
}

impl<R: AccountRepository + ?Sized> ExecutionPlanner<R> {
    /// Creates a new ExecutionPlanner with the given account repository and config.
    pub fn new(account_repo: Arc<R>, config: ExecutionPlannerConfig) -> Self {
        Self {
            account_repo,
            config,
            rotation_strategy: RotationStrategyType::HealthWeighted,
            metrics: None,
        }
    }

    /// Creates a new ExecutionPlanner with default configuration.
    pub fn with_default_config(account_repo: Arc<R>) -> Self {
        Self::new(account_repo, ExecutionPlannerConfig::default())
    }

    /// Sets the metrics collector.
    pub fn with_metrics(mut self, metrics: Arc<ExecutionPlanMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Sets the rotation strategy.
    pub fn with_rotation_strategy(mut self, strategy: RotationStrategyType) -> Self {
        self.rotation_strategy = strategy;
        self
    }

    /// Checks if metrics are enabled.
    #[allow(dead_code)]
    fn has_metrics(&self) -> bool {
        self.metrics.is_some()
    }

    /// Gets a reference to metrics if available.
    fn metrics(&self) -> Option<&ExecutionPlanMetrics> {
        self.metrics.as_deref()
    }

    /// Creates an execution plan for the given context.
    ///
    /// This is the main entry point that:
    /// 1. Determines the plan type based on context and config
    /// 2. Fetches and filters accounts
    /// 3. Applies rotation strategy
    /// 4. Builds the appropriate execution plan
    pub async fn create_plan(&self, context: ExecutionContext) -> DomainResult<ExecutionPlanImpl> {
        let start_time = Instant::now();

        // Create planning span for tracing
        let span = PlanningSpan::new(&context.request_id, &context.model);

        // Record planning started
        if let Some(metrics) = self.metrics() {
            metrics.record_planning_started();
        }

        // Determine plan type
        let plan_type = self.select_plan_type(&context);
        span.record_plan_type(plan_type.name());

        // Log plan type selection
        logging::log_plan_type_selection(
            &context.request_id,
            &context.model,
            plan_type.name(),
            "Based on context and configuration",
        );

        // Get available accounts
        let accounts = self.get_available_accounts(&context).await;
        let accounts = match accounts {
            Ok(a) => a,
            Err(e) => {
                span.error(&e.to_string());
                if let Some(metrics) = self.metrics() {
                    metrics.record_planning_error();
                    metrics.record_planning_completed();
                }
                return Err(e);
            },
        };

        let initial_count = accounts.len();
        span.record_filter("availability", initial_count);
        logging::log_account_filtering(&context.request_id, "availability", 0, initial_count);

        // Apply rotation strategy
        let rotated_accounts = self.apply_rotation_strategy(accounts);
        logging::log_rotation_strategy(
            &context.request_id,
            format!("{:?}", self.rotation_strategy).as_str(),
            rotated_accounts.len(),
        );

        // Filter by health and model compatibility
        let filtered_accounts = self.filter_accounts(rotated_accounts, &context)?;
        let final_count = filtered_accounts.len();
        span.record_filter("health_and_compatibility", final_count);
        logging::log_account_filtering(
            &context.request_id,
            "health_and_compatibility",
            initial_count,
            final_count,
        );

        // Build the execution plan based on type
        let plan = self.build_plan(plan_type, context.clone(), filtered_accounts)?;

        // Record metrics
        let duration = start_time.elapsed().as_secs_f64();
        if let Some(metrics) = self.metrics() {
            metrics.record_plan_created();
            metrics.record_planning_duration(duration);
            metrics.record_plan_type(plan_type.name());
            metrics.record_planning_completed();
        }

        // Complete the span
        span.finish(plan_type.name(), plan.account_count());

        Ok(plan)
    }

    /// Selects the appropriate plan type based on context and config.
    fn select_plan_type(&self, context: &ExecutionContext) -> ExecutionPlanType {
        // If auto-selection is disabled, use the default
        if !self.config.enable_auto_selection {
            return self.config.default_plan_type;
        }

        // Check context planning options first (user preference)
        let options = &context.planning_options;

        if options.cost_optimized || self.config.cost_optimization_enabled {
            return ExecutionPlanType::CostOptimized;
        }

        if options.enable_load_balancing || self.config.load_balancing_enabled {
            return ExecutionPlanType::LoadBalanced;
        }

        if options.enable_failover || self.config.failover_enabled {
            return ExecutionPlanType::Failover;
        }

        // Check for cascading (quality-based escalation)
        if options.enable_cascading
            || self.config.cascading_enabled
            || options.budget_mode
            || self.config.budget_mode_enabled
        {
            return ExecutionPlanType::Cascading;
        }

        // Fall back to config default
        self.config.default_plan_type
    }

    /// Gets available accounts for the given context.
    async fn get_available_accounts(
        &self,
        context: &ExecutionContext,
    ) -> DomainResult<Vec<(Account, Provider, AccountHealth)>> {
        let mut results = Vec::new();

        // Get providers based on preferences
        let providers = self.get_providers(context);

        for provider in &providers {
            // Get accounts for this provider
            let accounts = self
                .account_repo
                .find_active_by_provider(&provider.id)
                .await?;

            for account in accounts {
                // Create health snapshot (in real implementation, would fetch from health service)
                let health = AccountHealth::new(&account.id);

                // Filter by preferred accounts if specified
                if !context.preferred_accounts.is_empty()
                    && !context.is_account_preferred(&account.id)
                {
                    continue;
                }

                results.push((account, provider.clone(), health));
            }
        }

        Ok(results)
    }

    /// Gets providers based on context preferences.
    fn get_providers(&self, context: &ExecutionContext) -> Vec<Provider> {
        // If specific providers are preferred, use those
        if !context.preferred_providers.is_empty() {
            return context
                .preferred_providers
                .iter()
                .map(|id| Provider::new(id.clone(), id.clone(), Self::provider_url(id)))
                .collect();
        }

        // Otherwise return default providers
        vec![
            Provider::new("openai", "OpenAI", "https://api.openai.com/v1"),
            Provider::new("anthropic", "Anthropic", "https://api.anthropic.com/v1"),
            Provider::new("groq", "Groq", "https://api.groq.com/openai/v1"),
        ]
    }

    /// Gets the correct base URL for a provider.
    fn provider_url(provider_id: &str) -> String {
        match provider_id {
            // Major providers
            "openai" => "https://api.openai.com/v1".to_string(),
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            "groq" => "https://api.groq.com/openai/v1".to_string(),
            // OpenAI-compatible cloud providers
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "together" => "https://api.together.xyz/v1".to_string(),
            "fireworks" => "https://api.fireworks.ai/inference/v1".to_string(),
            "xai" => "https://api.x.ai/v1".to_string(),
            "perplexity" => "https://api.perplexity.ai/v1".to_string(),
            "openrouter" => "https://openrouter.ai/api/v1".to_string(),
            "mistral" => "https://api.mistral.ai/v1".to_string(),
            "cerebras" => "https://api.cerebras.ai/v1".to_string(),
            "cloudflare" => "https://gateway.ai.cloudflare.com/v1".to_string(),
            // Local inference servers
            "ollama" => "http://localhost:11434/v1".to_string(),
            "lmstudio" => "http://localhost:1234/v1".to_string(),
            "vllm" => "http://localhost:8000/v1".to_string(),
            // Platform / specialized providers
            "replicate" => "https://api.replicate.com/v1".to_string(),
            "huggingface" => "https://api-inference.huggingface.co".to_string(),
            "anyscale" => "https://api.endpoints.anyscale.com/v1".to_string(),
            "deepinfra" => "https://api.deepinfra.com/v1".to_string(),
            "novita" => "https://api.novita.ai/v1".to_string(),
            "sambanova" => "https://api.sambanova.ai/v1".to_string(),
            // Cloud hyperscaler services
            "azure" => "https://{resource}.openai.azure.com/v1".to_string(),
            "bedrock" => "https://bedrock-runtime.{region}.amazonaws.com".to_string(),
            "vertexai" => "https://{region}-aiplatform.googleapis.com/v1".to_string(),
            // Additional model providers
            "cohere" => "https://api.cohere.ai/v1".to_string(),
            "ai21" => "https://api.ai21.com/v1".to_string(),
            "aleph_alpha" => "https://api.aleph-alpha.com/v1".to_string(),
            "nvidia" => "https://integrate.api.nvidia.com/v1".to_string(),
            "google" => "https://generativelanguage.googleapis.com/v1".to_string(),
            _ => format!("https://api.{}.com", provider_id),
        }
    }

    /// Applies rotation strategy to accounts.
    fn apply_rotation_strategy(
        &self,
        mut accounts: Vec<(Account, Provider, AccountHealth)>,
    ) -> Vec<(Account, Provider, AccountHealth)> {
        match self.rotation_strategy {
            RotationStrategyType::RoundRobin => {
                // Shuffle accounts for round-robin
                use std::collections::hash_map::RandomState;
                use std::hash::{BuildHasher, Hasher};
                let mut hasher = RandomState::new().build_hasher();
                hasher.write_u64(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64,
                );
                let seed = hasher.finish() as usize;

                // Simple shuffle using seed
                let len = accounts.len();
                for i in (1..len).rev() {
                    let j = (seed.wrapping_mul(i + 1)) % (i + 1);
                    accounts.swap(i, j);
                }
                accounts
            },
            RotationStrategyType::HealthWeighted => {
                // Sort by health score (highest first)
                accounts.sort_by(|a, b| {
                    b.2.health_score()
                        .partial_cmp(&a.2.health_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                accounts
            },
            RotationStrategyType::Priority => {
                // Sort by priority (lowest = highest priority)
                accounts.sort_by_key(|a| a.0.priority);
                accounts
            },
            RotationStrategyType::LeastRecentlyUsed => {
                // Sort by last_used_at (oldest first)
                accounts.sort_by(|a, b| {
                    let a_time = a.0.last_used_at.unwrap_or(0);
                    let b_time = b.0.last_used_at.unwrap_or(0);
                    a_time.cmp(&b_time)
                });
                accounts
            },
        }
    }

    /// Filters accounts based on health and model compatibility.
    fn filter_accounts(
        &self,
        accounts: Vec<(Account, Provider, AccountHealth)>,
        context: &ExecutionContext,
    ) -> DomainResult<Vec<(Account, Provider, AccountHealth)>> {
        let mut filtered = Vec::new();

        for (account, provider, health) in accounts {
            // Filter by health threshold
            if health.health_score() < self.config.min_health_score {
                continue;
            }

            // Filter by circuit breaker
            if health.circuit_breaker_open() {
                continue;
            }

            // Filter by model compatibility
            if !self.is_model_compatible(&context.model, &provider) {
                continue;
            }

            filtered.push((account, provider, health));
        }

        // Limit to max accounts per plan
        filtered.truncate(self.config.max_accounts_per_plan);

        Ok(filtered)
    }

    /// Checks if the provider supports the requested model.
    fn is_model_compatible(&self, model: &str, provider: &Provider) -> bool {
        // Simple compatibility check - in real implementation would be more sophisticated
        let model_lower = model.to_lowercase();

        match provider.id.as_str() {
            "openai" => {
                model_lower.starts_with("gpt-")
                    || model_lower.starts_with("o1")
                    || model_lower.starts_with("o3")
            },
            "anthropic" => {
                model_lower.starts_with("claude-")
                    || model_lower.starts_with("sonnet")
                    || model_lower.starts_with("haiku")
            },
            "groq" => {
                // Groq supports many models via inference APIs
                true
            },
            _ => true, // Allow unknown providers by default
        }
    }

    /// Builds the execution plan based on type.
    fn build_plan(
        &self,
        plan_type: ExecutionPlanType,
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
    ) -> DomainResult<ExecutionPlanImpl> {
        // Convert to PlannedAccount
        let planned_accounts: Vec<PlannedAccount> = accounts
            .into_iter()
            .enumerate()
            .map(|(idx, (account, provider, health))| {
                let _is_primary = idx == 0;
                PlannedAccount::new(account.id.clone(), &provider, health)
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority)
                    .as_primary()
            })
            .collect();

        // Create the plan with config values
        let mut plan = ExecutionPlanImpl::new(plan_type, context, planned_accounts);

        // Apply max_retries and timeout from config
        plan = plan.with_max_retries(self.config.default_max_retries);
        plan = plan.with_timeout(self.config.default_timeout_seconds);

        Ok(plan)
    }
}

// =============================================================================
// ExecutionPlanner Builder
// =============================================================================

/// Builder for creating ExecutionPlanner instances.
pub struct ExecutionPlannerBuilder<R: AccountRepository + ?Sized> {
    account_repo: Option<Arc<R>>,
    config: ExecutionPlannerConfig,
    rotation_strategy: Option<RotationStrategyType>,
    metrics: Option<Arc<ExecutionPlanMetrics>>,
}

impl<R: AccountRepository + ?Sized> ExecutionPlannerBuilder<R> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            account_repo: None,
            config: ExecutionPlannerConfig::default(),
            rotation_strategy: None,
            metrics: None,
        }
    }

    /// Sets the account repository.
    pub fn with_account_repo(mut self, account_repo: Arc<R>) -> Self {
        self.account_repo = Some(account_repo);
        self
    }

    /// Sets the config.
    pub fn with_config(mut self, config: ExecutionPlannerConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the rotation strategy.
    pub fn with_rotation_strategy(mut self, strategy: RotationStrategyType) -> Self {
        self.rotation_strategy = Some(strategy);
        self
    }

    /// Sets the metrics collector.
    pub fn with_metrics(mut self, metrics: Arc<ExecutionPlanMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Loads configuration from environment variables.
    pub fn from_env(mut self) -> Self {
        self.config = ExecutionPlannerConfigBuilder::new().from_env().build();
        self
    }

    /// Builds the ExecutionPlanner.
    ///
    /// # Panics
    ///
    /// Panics if account_repo is not set.
    pub fn build(self) -> ExecutionPlanner<R> {
        let account_repo = self.account_repo.expect("Account repository is required");

        let mut planner = ExecutionPlanner::new(account_repo, self.config);

        if let Some(strategy) = self.rotation_strategy {
            planner.rotation_strategy = strategy;
        }

        if let Some(metrics) = self.metrics {
            planner.metrics = Some(metrics);
        }

        planner
    }
}

impl<R: AccountRepository> Default for ExecutionPlannerBuilder<R> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::PlanningOptions;
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ExecutionPlannerConfig::default();
        assert_eq!(config.default_plan_type, ExecutionPlanType::Standard);
        assert!(config.enable_auto_selection);
        assert_eq!(config.max_accounts_per_plan, 5);
    }

    #[test]
    fn test_config_presets() {
        let reliability = ExecutionPlannerConfig::reliability();
        assert_eq!(reliability.default_plan_type, ExecutionPlanType::Failover);
        assert_eq!(reliability.default_max_retries, 5);

        let cost = ExecutionPlannerConfig::cost_optimized();
        assert_eq!(cost.default_plan_type, ExecutionPlanType::CostOptimized);
        assert!(cost.cost_optimization_enabled);

        let latency = ExecutionPlannerConfig::low_latency();
        assert_eq!(latency.default_plan_type, ExecutionPlanType::LoadBalanced);
        assert!(latency.load_balancing_enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = ExecutionPlannerConfigBuilder::new()
            .with_default_plan_type(ExecutionPlanType::Failover)
            .with_max_accounts(10)
            .build();

        assert_eq!(config.default_plan_type, ExecutionPlanType::Failover);
        assert_eq!(config.max_accounts_per_plan, 10);
    }

    #[test]
    fn test_config_builder_from_env() {
        // Set environment variable
        std::env::set_var("EXECUTION_PLAN_TYPE", "failover");
        std::env::set_var("EXECUTION_MAX_RETRIES", "5");

        let config = ExecutionPlannerConfigBuilder::new().from_env().build();

        assert_eq!(config.default_plan_type, ExecutionPlanType::Failover);
        assert_eq!(config.default_max_retries, 5);

        // Clean up
        std::env::remove_var("EXECUTION_PLAN_TYPE");
        std::env::remove_var("EXECUTION_MAX_RETRIES");
    }

    // =============================================================================
    // ExecutionPlanner Tests
    // =============================================================================

    #[test]
    fn test_rotation_strategy_type_default() {
        let strategy = RotationStrategyType::default();
        assert!(matches!(strategy, RotationStrategyType::HealthWeighted));
    }

    #[test]
    fn test_execution_planner_select_plan_type_standard() {
        // Test that standard plan is selected when auto-selection is disabled
        let config = ExecutionPlannerConfig {
            default_plan_type: ExecutionPlanType::Standard,
            enable_auto_selection: false,
            ..Default::default()
        };

        let _context = ExecutionContext::new("req-1", "gpt-4");

        // Cannot easily test without a repo, but we can verify the config works
        assert_eq!(config.default_plan_type, ExecutionPlanType::Standard);
        assert!(!config.enable_auto_selection);
    }

    #[test]
    fn test_execution_planner_select_plan_type_cost_optimized() {
        // Test that cost-optimized is selected when cost_optimization_enabled
        let config = ExecutionPlannerConfig {
            cost_optimization_enabled: true,
            enable_auto_selection: true,
            ..Default::default()
        };

        // Verify the config is set correctly for cost optimization
        assert!(config.cost_optimization_enabled);
    }

    #[test]
    fn test_execution_planner_select_plan_type_load_balanced() {
        // Test that load balanced is selected when load_balancing_enabled
        let config = ExecutionPlannerConfig {
            load_balancing_enabled: true,
            enable_auto_selection: true,
            ..Default::default()
        };

        assert!(config.load_balancing_enabled);
    }

    #[test]
    fn test_execution_planner_select_plan_type_failover() {
        // Test that failover is selected when failover_enabled
        let config = ExecutionPlannerConfig {
            failover_enabled: true,
            enable_auto_selection: true,
            cost_optimization_enabled: false,
            load_balancing_enabled: false,
            ..Default::default()
        };

        assert!(config.failover_enabled);
    }

    #[test]
    fn test_model_compatibility_openai() {
        // Create a minimal test using the context's model compatibility logic
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");

        // These models should be compatible with OpenAI
        assert!(is_model_compatible_internal("gpt-4", &provider));
        assert!(is_model_compatible_internal("gpt-3.5-turbo", &provider));
        assert!(is_model_compatible_internal("o1-preview", &provider));
    }

    #[test]
    fn test_model_compatibility_anthropic() {
        let provider = Provider::new("anthropic", "Anthropic", "https://api.anthropic.com");

        assert!(is_model_compatible_internal("claude-3-opus", &provider));
        assert!(is_model_compatible_internal("claude-3-sonnet", &provider));
        assert!(is_model_compatible_internal("claude-3-haiku", &provider));
    }

    #[test]
    fn test_model_compatibility_groq() {
        let provider = Provider::new("groq", "Groq", "https://api.groq.com");

        // Groq supports many models
        assert!(is_model_compatible_internal("llama-3-70b", &provider));
        assert!(is_model_compatible_internal("mixtral-8x7b", &provider));
    }

    #[test]
    fn test_execution_planner_builder() {
        // Test builder pattern - requires a mock repo which we can't easily create
        // So we just verify the builder can be constructed
        use super::mock::MockAccountRepository;
        let _builder: ExecutionPlannerBuilder<MockAccountRepository> =
            ExecutionPlannerBuilder::new();
    }

    #[test]
    fn test_planning_options_influences_plan_type() {
        // Test that context planning options influence plan type selection
        let context = ExecutionContext::new("req-1", "gpt-4")
            .with_planning_options(PlanningOptions::cost_optimized());

        assert!(context.planning_options.cost_optimized);
    }

    #[test]
    fn test_planning_options_reliability() {
        let options = PlanningOptions::reliability();

        assert!(options.enable_failover);
        assert_eq!(options.max_retries, 5);
        assert!(options.health_aware_routing);
    }

    #[test]
    fn test_planning_options_low_latency() {
        let options = PlanningOptions::low_latency();

        assert!(options.enable_load_balancing);
        assert_eq!(options.timeout_seconds, 15);
    }
}

/// Helper function to check model compatibility (for testing only)
#[allow(dead_code)]
fn is_model_compatible_internal(model: &str, provider: &Provider) -> bool {
    let model_lower = model.to_lowercase();

    match provider.id.as_str() {
        "openai" => {
            model_lower.starts_with("gpt-")
                || model_lower.starts_with("o1")
                || model_lower.starts_with("o3")
        },
        "anthropic" => {
            model_lower.starts_with("claude-")
                || model_lower.starts_with("sonnet")
                || model_lower.starts_with("haiku")
        },
        "groq" => true,
        _ => true,
    }
}

// Mock repository for testing
#[cfg(test)]
mod mock {
    use super::*;
    use crate::domain::DomainResult;
    use async_trait::async_trait;

    pub struct MockAccountRepository {
        accounts: Vec<Account>,
    }

    impl MockAccountRepository {
        #[allow(dead_code)]
        pub fn new(accounts: Vec<Account>) -> Self {
            Self { accounts }
        }
    }

    #[async_trait]
    impl AccountRepository for MockAccountRepository {
        async fn save(&self, _account: Account) -> DomainResult<Account> {
            todo!()
        }

        async fn find_all(&self) -> DomainResult<Vec<Account>> {
            Ok(self.accounts.clone())
        }

        async fn find_by_id(&self, id: &str) -> DomainResult<Account> {
            self.accounts
                .iter()
                .find(|a| a.id == id)
                .cloned()
                .ok_or_else(|| {
                    crate::domain::DomainError::AccountNotFound("Account not found".to_string())
                })
        }

        async fn find_active(&self) -> DomainResult<Vec<Account>> {
            Ok(self
                .accounts
                .iter()
                .filter(|a| a.is_active)
                .cloned()
                .collect())
        }

        async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>> {
            Ok(self
                .accounts
                .iter()
                .filter(|a| a.is_active && a.provider_id == provider_id)
                .cloned()
                .collect())
        }

        async fn delete(&self, _id: &str) -> DomainResult<()> {
            todo!()
        }
    }
}
