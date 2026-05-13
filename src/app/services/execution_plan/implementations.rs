//! Execution Plan Implementations
//!
//! Concrete implementations of different execution strategies.

use std::sync::Arc;

use crate::domain::entities::{Account, AccountHealth, Provider};
use crate::domain::traits::AccountRepository;
use crate::domain::{DomainError, DomainResult};

use super::types::{ExecutionPlanType, PlannedAccount};
use super::{
    ExecutionContext, ExecutionOutcome, ExecutionPlan, ExecutionPlanImpl, ExecutionPlanStatus,
};
use crate::app::services::execution_plan::CascadingExecutionPlan;
use crate::app::services::quality::{HeuristicQualityEvaluator, QualityConfig, QualityGate};

/// Provider pricing information for cost optimization.
#[derive(Debug, Clone)]
pub struct ProviderPricing {
    /// Provider ID
    pub provider_id: String,
    /// Price per 1M input tokens
    pub input_price_per_1m: f64,
    /// Price per 1M output tokens
    pub output_price_per_1m: f64,
}

impl ProviderPricing {
    /// Returns estimated cost for given token counts.
    pub fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.input_price_per_1m
            + (output_tokens as f64 / 1_000_000.0) * self.output_price_per_1m
    }
}

/// StandardExecutionPlan - Basic single account execution with health check.
///
/// This plan selects a single account for execution and verifies its health
/// before proceeding. It provides the simplest execution strategy with minimal
/// overhead.
#[derive(Debug)]
pub struct StandardExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,
}

impl StandardExecutionPlan {
    /// Creates a new StandardExecutionPlan from accounts and context.
    pub fn new(
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
    ) -> Self {
        let planned_accounts: Vec<PlannedAccount> = accounts
            .into_iter()
            .enumerate()
            .map(|(idx, (account, provider, health))| {
                PlannedAccount::new(account.id.clone(), &provider, health)
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority)
                    .as_primary()
            })
            .collect();

        let inner = ExecutionPlanImpl::new(ExecutionPlanType::Standard, context, planned_accounts)
            .with_max_retries(3)
            .with_timeout(60);

        Self { inner }
    }

    /// Checks if the primary account is healthy enough to use.
    pub fn is_primary_healthy(&self) -> bool {
        self.inner
            .primary_account()
            .map(|a| a.is_healthy())
            .unwrap_or(false)
    }

    /// Gets the health score of the primary account.
    pub fn primary_health_score(&self) -> Option<f64> {
        self.inner.primary_account().map(|a| a.health_score())
    }
}

impl ExecutionPlan for StandardExecutionPlan {
    fn plan_type(&self) -> ExecutionPlanType {
        self.inner.plan_type()
    }

    fn planned_accounts(&self) -> &[PlannedAccount] {
        self.inner.planned_accounts()
    }

    fn status(&self) -> ExecutionPlanStatus {
        self.inner.status()
    }

    fn context(&self) -> &ExecutionContext {
        self.inner.context()
    }

    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }

    fn timeout_seconds(&self) -> u32 {
        self.inner.timeout_seconds()
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.inner.outcome()
    }

    fn error_message(&self) -> Option<&str> {
        self.inner.error_message()
    }

    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.inner.update_status(status);
    }

    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.inner.set_outcome(outcome);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.inner.set_error(message);
    }
}

/// FailoverExecutionPlan - Pre-planned failover path with ordered accounts.
///
/// This plan maintains a pre-ordered list of accounts to try in sequence.
/// If the primary account fails, it automatically proceeds to the next account
/// in the failover chain.
#[derive(Debug)]
pub struct FailoverExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,
}

impl FailoverExecutionPlan {
    /// Creates a new FailoverExecutionPlan from accounts and context.
    ///
    /// Accounts are ordered by priority for failover sequence.
    pub fn new(
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
    ) -> Self {
        let planned_accounts: Vec<PlannedAccount> = accounts
            .into_iter()
            .enumerate()
            .map(|(idx, (account, provider, health))| {
                let is_primary = idx == 0;
                let mut planned = PlannedAccount::new(account.id.clone(), &provider, health)
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority);

                if is_primary {
                    planned.is_primary = true;
                    planned.is_fallback = false;
                } else {
                    planned.is_primary = false;
                    planned.is_fallback = true;
                }
                planned
            })
            .collect();

        let inner = ExecutionPlanImpl::new(
            ExecutionPlanType::Failover,
            context.clone(),
            planned_accounts,
        )
        .with_max_retries(context.planning_options.max_retries)
        .with_timeout(context.planning_options.timeout_seconds);

        Self { inner }
    }

    /// Returns the failover chain as account IDs.
    pub fn failover_chain(&self) -> Vec<&str> {
        self.inner
            .planned_accounts()
            .iter()
            .map(|a| a.account_id.as_str())
            .collect()
    }

    /// Returns the number of fallback accounts available.
    pub fn fallback_count(&self) -> usize {
        self.inner.fallback_accounts().len()
    }
}

impl ExecutionPlan for FailoverExecutionPlan {
    fn plan_type(&self) -> ExecutionPlanType {
        self.inner.plan_type()
    }

    fn planned_accounts(&self) -> &[PlannedAccount] {
        self.inner.planned_accounts()
    }

    fn status(&self) -> ExecutionPlanStatus {
        self.inner.status()
    }

    fn context(&self) -> &ExecutionContext {
        self.inner.context()
    }

    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }

    fn timeout_seconds(&self) -> u32 {
        self.inner.timeout_seconds()
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.inner.outcome()
    }

    fn error_message(&self) -> Option<&str> {
        self.inner.error_message()
    }

    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.inner.update_status(status);
    }

    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.inner.set_outcome(outcome);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.inner.set_error(message);
    }
}

/// LoadBalancedExecutionPlan - Distributes load based on health/latency.
///
/// This plan selects accounts based on their health scores and latency,
/// distributing requests across multiple healthy accounts to balance load.
#[derive(Debug)]
pub struct LoadBalancedExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,

    /// Health-based weights for load distribution
    weights: Vec<f64>,
}

impl LoadBalancedExecutionPlan {
    /// Creates a new LoadBalancedExecutionPlan from accounts and context.
    ///
    /// Accounts are weighted based on health scores for load distribution.
    pub fn new(
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
    ) -> Self {
        let planned_accounts: Vec<PlannedAccount> = accounts
            .iter()
            .enumerate()
            .map(|(idx, (account, provider, health))| {
                PlannedAccount::new(account.id.clone(), provider, health.clone())
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority)
            })
            .collect();

        // Calculate weights based on health scores
        let weights: Vec<f64> = accounts
            .iter()
            .map(|(_, _, health)| health.health_score())
            .collect();

        let inner =
            ExecutionPlanImpl::new(ExecutionPlanType::LoadBalanced, context, planned_accounts)
                .with_max_retries(2)
                .with_timeout(45);

        Self { inner, weights }
    }

    /// Returns the weight for a specific account index.
    pub fn weight_for(&self, index: usize) -> f64 {
        self.weights.get(index).copied().unwrap_or(0.0)
    }

    /// Returns total weight across all accounts.
    pub fn total_weight(&self) -> f64 {
        self.weights.iter().sum()
    }

    /// Selects an account index based on weighted random selection.
    pub fn select_by_weight(&self) -> Option<usize> {
        if self.weights.is_empty() {
            return None;
        }

        let total = self.total_weight();
        if total <= 0.0 {
            return Some(0);
        }

        use std::time::{SystemTime, UNIX_EPOCH};
        let random = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64
            % total;

        let mut cumulative = 0.0;
        for (idx, &weight) in self.weights.iter().enumerate() {
            cumulative += weight;
            if random <= cumulative {
                return Some(idx);
            }
        }

        Some(self.weights.len() - 1)
    }

    /// Returns accounts sorted by health (best first).
    pub fn accounts_by_health(&self) -> Vec<&PlannedAccount> {
        let mut accounts: Vec<_> = self.inner.planned_accounts().iter().collect();
        accounts.sort_by(|a, b| {
            b.health_score()
                .partial_cmp(&a.health_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        accounts
    }

    /// Returns accounts sorted by latency (fastest first).
    pub fn accounts_by_latency(&self) -> Vec<&PlannedAccount> {
        let mut accounts: Vec<_> = self.inner.planned_accounts().iter().collect();
        accounts.sort_by(|a, b| {
            a.health_snapshot
                .avg_latency_ms
                .partial_cmp(&b.health_snapshot.avg_latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        accounts
    }
}

impl ExecutionPlan for LoadBalancedExecutionPlan {
    fn plan_type(&self) -> ExecutionPlanType {
        self.inner.plan_type()
    }

    fn planned_accounts(&self) -> &[PlannedAccount] {
        self.inner.planned_accounts()
    }

    fn status(&self) -> ExecutionPlanStatus {
        self.inner.status()
    }

    fn context(&self) -> &ExecutionContext {
        self.inner.context()
    }

    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }

    fn timeout_seconds(&self) -> u32 {
        self.inner.timeout_seconds()
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.inner.outcome()
    }

    fn error_message(&self) -> Option<&str> {
        self.inner.error_message()
    }

    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.inner.update_status(status);
    }

    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.inner.set_outcome(outcome);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.inner.set_error(message);
    }
}

/// CostOptimizedExecutionPlan - Selects cheapest provider for model.
///
/// This plan considers pricing information to select the most cost-effective
/// provider for the requested model while meeting quality requirements.
#[derive(Debug)]
pub struct CostOptimizedExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,

    /// Pricing information used for selection
    pricing: Vec<ProviderPricing>,
}

impl CostOptimizedExecutionPlan {
    /// Creates a new CostOptimizedExecutionPlan from accounts, providers, and context.
    pub fn new(
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
        pricing: Vec<ProviderPricing>,
    ) -> Self {
        // Sort accounts by provider cost
        let mut account_data: Vec<_> = accounts
            .into_iter()
            .map(|(account, provider, health)| {
                let cost = pricing
                    .iter()
                    .find(|p| p.provider_id == provider.id)
                    .map(|p| p.estimate_cost(1000, 1000)) // Estimate for typical request
                    .unwrap_or(f64::MAX);

                (account, provider, health, cost)
            })
            .collect();

        // Sort by cost (cheapest first), then by health
        account_data.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        let planned_accounts: Vec<PlannedAccount> = account_data
            .iter()
            .enumerate()
            .map(|(idx, (account, provider, health, _))| {
                PlannedAccount::new(account.id.clone(), provider, health.clone())
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority)
            })
            .collect();

        let inner =
            ExecutionPlanImpl::new(ExecutionPlanType::CostOptimized, context, planned_accounts)
                .with_max_retries(1)
                .with_timeout(30);

        Self { inner, pricing }
    }

    /// Returns the cheapest provider for the given model.
    pub fn cheapest_provider(&self) -> Option<&ProviderPricing> {
        self.pricing.iter().min_by(|a, b| {
            a.input_price_per_1m
                .partial_cmp(&b.input_price_per_1m)
                .unwrap()
        })
    }

    /// Returns pricing for all providers.
    pub fn all_pricing(&self) -> &[ProviderPricing] {
        &self.pricing
    }
}

impl ExecutionPlan for CostOptimizedExecutionPlan {
    fn plan_type(&self) -> ExecutionPlanType {
        self.inner.plan_type()
    }

    fn planned_accounts(&self) -> &[PlannedAccount] {
        self.inner.planned_accounts()
    }

    fn status(&self) -> ExecutionPlanStatus {
        self.inner.status()
    }

    fn context(&self) -> &ExecutionContext {
        self.inner.context()
    }

    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }

    fn timeout_seconds(&self) -> u32 {
        self.inner.timeout_seconds()
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.inner.outcome()
    }

    fn error_message(&self) -> Option<&str> {
        self.inner.error_message()
    }

    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.inner.update_status(status);
    }

    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.inner.set_outcome(outcome);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.inner.set_error(message);
    }
}

/// Builds execution plans from available accounts and providers.
pub struct ExecutionPlanBuilder<R: AccountRepository> {
    account_repo: Arc<R>,
    pricing: Vec<ProviderPricing>,
}

impl<R: AccountRepository> ExecutionPlanBuilder<R> {
    /// Creates a new ExecutionPlanBuilder.
    pub fn new(account_repo: Arc<R>) -> Self {
        Self {
            account_repo,
            pricing: Vec::new(),
        }
    }

    /// Sets pricing information for cost optimization.
    pub fn with_pricing(mut self, pricing: Vec<ProviderPricing>) -> Self {
        self.pricing = pricing;
        self
    }

    /// Builds a StandardExecutionPlan.
    pub async fn build_standard(
        &self,
        context: ExecutionContext,
    ) -> Result<StandardExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;

        Ok(StandardExecutionPlan::new(context, accounts))
    }

    /// Builds a FailoverExecutionPlan.
    pub async fn build_failover(
        &self,
        context: ExecutionContext,
    ) -> Result<FailoverExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;

        Ok(FailoverExecutionPlan::new(context, accounts))
    }

    /// Builds a LoadBalancedExecutionPlan.
    pub async fn build_load_balanced(
        &self,
        context: ExecutionContext,
    ) -> Result<LoadBalancedExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;

        Ok(LoadBalancedExecutionPlan::new(context, accounts))
    }

    /// Builds a CostOptimizedExecutionPlan.
    pub async fn build_cost_optimized(
        &self,
        context: ExecutionContext,
    ) -> Result<CostOptimizedExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;

        Ok(CostOptimizedExecutionPlan::new(
            context,
            accounts,
            self.pricing.clone(),
        ))
    }

    /// Builds a CascadingExecutionPlan.
    pub async fn build_cascading(
        &self,
        context: ExecutionContext,
        model_ids: Vec<String>,
        quality_config: Option<QualityConfig>,
        quality_gate: Option<Arc<dyn QualityGate>>,
    ) -> Result<CascadingExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;

        // Use default quality config if none provided
        let config = quality_config.unwrap_or_default();

        // Use default heuristic evaluator if none provided
        let gate = quality_gate.unwrap_or_else(|| Arc::new(HeuristicQualityEvaluator::new()));

        Ok(CascadingExecutionPlan::new(
            context,
            accounts,
            self.pricing.clone(),
            model_ids,
            config,
            gate,
        ))
    }

    /// Gets accounts for the given context.
    async fn get_accounts_for_context(
        &self,
        context: &ExecutionContext,
    ) -> DomainResult<Vec<(Account, Provider, AccountHealth)>> {
        let mut results = Vec::new();

        // Helper function to get provider URL
        fn get_provider_url(provider_id: &str) -> String {
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

        // Get all providers from preferences or all available
        let providers = if context.preferred_providers.is_empty() {
            // Get all providers - simplified for now
            vec![
                Provider::new("openai", "OpenAI", "https://api.openai.com/v1"),
                Provider::new("anthropic", "Anthropic", "https://api.anthropic.com/v1"),
                Provider::new("groq", "Groq", "https://api.groq.com/openai/v1"),
            ]
        } else {
            context
                .preferred_providers
                .iter()
                .map(|id| Provider::new(id.clone(), id.clone(), get_provider_url(id)))
                .collect()
        };

        for provider in &providers {
            let accounts = self
                .account_repo
                .find_active_by_provider(&provider.id)
                .await?;

            for account in accounts {
                // Get or create health (simplified - in real impl would get from health service)
                let health = AccountHealth::new(&account.id);

                results.push((account, provider.clone(), health));
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Account, Provider};

    fn create_test_context() -> ExecutionContext {
        ExecutionContext::new("test-req-1", "gpt-4")
    }

    fn create_test_accounts() -> Vec<(Account, Provider, AccountHealth)> {
        vec![
            (
                Account::new_api_key("acc-1", "openai", "sk-test-1"),
                Provider::new("openai", "OpenAI", "https://api.openai.com"),
                AccountHealth::new("acc-1"),
            ),
            (
                Account::new_api_key("acc-2", "anthropic", "sk-test-2"),
                Provider::new("anthropic", "Anthropic", "https://api.anthropic.com"),
                AccountHealth::new("acc-2"),
            ),
        ]
    }

    #[test]
    fn test_standard_execution_plan() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = StandardExecutionPlan::new(ctx.clone(), accounts);

        assert_eq!(plan.plan_type(), ExecutionPlanType::Standard);
        assert!(plan.has_accounts());
        assert!(plan.primary_account().is_some());
    }

    #[test]
    fn test_standard_execution_plan_health() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = StandardExecutionPlan::new(ctx, accounts);

        assert!(plan.is_primary_healthy());
        assert!(plan.primary_health_score().is_some());
    }

    #[test]
    fn test_failover_execution_plan() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = FailoverExecutionPlan::new(ctx.clone(), accounts);

        assert_eq!(plan.plan_type(), ExecutionPlanType::Failover);
        assert!(plan.has_accounts());
        assert_eq!(plan.fallback_count(), 1);
    }

    #[test]
    fn test_failover_chain() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = FailoverExecutionPlan::new(ctx, accounts);

        let chain = plan.failover_chain();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "acc-1");
        assert_eq!(chain[1], "acc-2");
    }

    #[test]
    fn test_load_balanced_execution_plan() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = LoadBalancedExecutionPlan::new(ctx.clone(), accounts);

        assert_eq!(plan.plan_type(), ExecutionPlanType::LoadBalanced);
        assert!(plan.total_weight() > 0.0);
    }

    #[test]
    fn test_load_balanced_weight_selection() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();

        let plan = LoadBalancedExecutionPlan::new(ctx, accounts);

        // Should always select an account
        assert!(plan.select_by_weight().is_some());
    }

    #[test]
    fn test_cost_optimized_execution_plan() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();
        let pricing = vec![
            ProviderPricing {
                provider_id: "openai".to_string(),
                input_price_per_1m: 10.0,
                output_price_per_1m: 30.0,
            },
            ProviderPricing {
                provider_id: "anthropic".to_string(),
                input_price_per_1m: 15.0,
                output_price_per_1m: 75.0,
            },
        ];

        let plan = CostOptimizedExecutionPlan::new(ctx.clone(), accounts, pricing);

        assert_eq!(plan.plan_type(), ExecutionPlanType::CostOptimized);
        assert!(plan.cheapest_provider().is_some());
    }

    #[test]
    fn test_cost_optimized_cheapest() {
        let ctx = create_test_context();
        let accounts = create_test_accounts();
        let pricing = vec![
            ProviderPricing {
                provider_id: "openai".to_string(),
                input_price_per_1m: 10.0,
                output_price_per_1m: 30.0,
            },
            ProviderPricing {
                provider_id: "anthropic".to_string(),
                input_price_per_1m: 15.0,
                output_price_per_1m: 75.0,
            },
        ];

        let plan = CostOptimizedExecutionPlan::new(ctx, accounts, pricing);

        let cheapest = plan.cheapest_provider().unwrap();
        assert_eq!(cheapest.provider_id, "openai");
    }

    #[test]
    fn test_provider_pricing_estimate() {
        let pricing = ProviderPricing {
            provider_id: "test".to_string(),
            input_price_per_1m: 10.0,
            output_price_per_1m: 30.0,
        };

        // 1000 input tokens = $0.01, 500 output tokens = $0.015
        let cost = pricing.estimate_cost(1000, 500);
        assert!((cost - 0.025).abs() < 0.001);
    }
}
