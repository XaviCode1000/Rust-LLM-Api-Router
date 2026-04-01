use std::sync::Arc;

use crate::app::services::execution_plan::execution::{ExecutionConfig, ExecutionResult};
use crate::app::services::execution_plan::types::{ExecutionPlanType, PlannedAccount};
use crate::app::services::execution_plan::{
    ExecutionContext, ExecutionOutcome, ExecutionPlan, ExecutionPlanImpl, ExecutionPlanStatus,
    ProviderPricing, QualityEvaluationSpan,
};
use crate::app::services::quality::evaluator::{QualityConfig, QualityGate};
use crate::domain::entities::{Account, AccountHealth, Provider};

/// Represents a single tier in a cascading execution plan.
#[derive(Debug, Clone)]
pub struct CascadingTier {
    /// The account for this tier
    pub account: PlannedAccount,
    /// The model ID to use for this tier
    pub model_id: String,
    /// The order of this tier (0-based, lower = tried first)
    pub tier_order: u32,
}

impl CascadingTier {
    /// Creates a new CascadingTier.
    ///
    /// # Arguments
    ///
    /// * `account` - The planned account for this tier
    /// * `model_id` - The model ID to use
    /// * `tier_order` - The order of this tier
    ///
    /// # Returns
    ///
    /// A new CascadingTier instance
    pub fn new(account: PlannedAccount, model_id: impl Into<String>, tier_order: u32) -> Self {
        Self {
            account,
            model_id: model_id.into(),
            tier_order,
        }
    }
}

/// CascadingExecutionPlan - Tries tiers in order of cost, escalating on quality failure.
///
/// This plan implements a cascading strategy where it starts with the cheapest
/// available tier and escalates to more expensive/higher quality tiers if the
/// response quality falls below acceptable thresholds.
#[derive(Debug)]
pub struct CascadingExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,
    /// Tiers to attempt in order (cheapest first)
    tiers: Vec<CascadingTier>,
    /// Quality configuration for evaluation
    quality_config: QualityConfig,
    /// Total cost in microdollars accumulated so far
    total_cost_microdollars: u64,
    /// Number of tiers that have been attempted
    tiers_attempted: u32,
    /// Quality gate for evaluating responses (reserved for future use when integrating with real LLM execution)
    #[allow(dead_code)]
    quality_gate: Arc<dyn QualityGate>,
}

impl CascadingExecutionPlan {
    /// Creates a new CascadingExecutionPlan from accounts, providers, pricing, and context.
    ///
    /// # Arguments
    ///
    /// * `context` - Execution context
    /// * `accounts` - Vector of (account, provider, health) tuples
    /// * `pricing` - Provider pricing information for cost sorting
    /// * `model_ids` - Model IDs to use for each tier (should match accounts length)
    /// * `quality_config` - Quality configuration for evaluation
    /// * `quality_gate` - Quality gate implementation to use
    ///
    /// # Returns
    ///
    /// A new CascadingExecutionPlan instance
    pub fn new(
        context: ExecutionContext,
        accounts: Vec<(Account, Provider, AccountHealth)>,
        pricing: Vec<ProviderPricing>,
        model_ids: Vec<String>,
        quality_config: QualityConfig,
        quality_gate: Arc<dyn QualityGate>,
    ) -> Self {
        // Validate that we have model IDs for each account
        assert_eq!(
            accounts.len(),
            model_ids.len(),
            "Number of model IDs must match number of accounts"
        );

        // Create account data with cost information for sorting
        let mut account_data: Vec<_> = accounts
            .into_iter()
            .zip(model_ids.into_iter())
            .enumerate()
            .map(|(idx, ((account, provider, health), model_id))| {
                let cost_per_request = pricing
                    .iter()
                    .find(|p| p.provider_id == provider.id)
                    .map(|p| {
                        // Estimate cost for a typical request (1000 input, 1000 output tokens)
                        (p.estimate_cost(1000, 1000) * 1_000_000.0) as u64 // Convert to microdollars
                    })
                    .unwrap_or(u64::MAX);

                (
                    account,
                    provider,
                    health,
                    model_id,
                    cost_per_request,
                    idx as u32,
                )
            })
            .collect();

        // Sort by cost (cheapest first), then by health score (best first)
        account_data.sort_by(|a, b| {
            a.4.cmp(&b.4).then_with(|| {
                b.2.health_score()
                    .partial_cmp(&a.2.health_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // Create tiers from sorted accounts
        let tiers: Vec<CascadingTier> = account_data
            .into_iter()
            .enumerate()
            .map(|(idx, (account, provider, health, model_id, _, _))| {
                let mut planned = PlannedAccount::new(account.id.clone(), &provider, health)
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority);

                // Mark only the first tier as primary initially
                if idx == 0 {
                    planned = planned.as_primary();
                } else {
                    planned = planned.as_fallback();
                }

                planned = planned.with_model_id(&model_id);

                CascadingTier::new(planned, model_id, idx as u32)
            })
            .collect();

        // Limit to max tiers from config
        let tiers: Vec<CascadingTier> = tiers
            .into_iter()
            .take(quality_config.max_tiers as usize)
            .collect();

        let inner = ExecutionPlanImpl::new(
            ExecutionPlanType::Cascading,
            context,
            tiers.iter().map(|t| t.account.clone()).collect(),
        )
        .with_max_retries(1) // Each tier gets one attempt
        .with_timeout(quality_config.per_tier_timeout_ms as u32 / 1000);

        Self {
            inner,
            tiers,
            quality_config,
            total_cost_microdollars: 0,
            tiers_attempted: 0,
            quality_gate,
        }
    }

    /// Returns the current tier being attempted.
    pub fn current_tier(&self) -> Option<&CascadingTier> {
        if (self.tiers_attempted as usize) < self.tiers.len() {
            Some(&self.tiers[self.tiers_attempted as usize])
        } else {
            None
        }
    }

    /// Escalates to the next tier in the cascade.
    ///
    /// # Returns
    ///
    /// True if there is a next tier available, false if we've exhausted all tiers
    pub fn escalate_to_next_tier(&mut self) -> bool {
        if self.tiers_attempted < self.tiers.len() as u32 {
            self.tiers_attempted += 1;

            // If we still have a valid tier, update the inner plan
            if self.current_tier().is_some() {
                // Reset the inner plan with the new tier as primary
                let mut planned_accounts: Vec<PlannedAccount> =
                    self.tiers.iter().map(|t| t.account.clone()).collect();

                // Mark current tier as primary, others as fallback
                for (idx, account) in planned_accounts.iter_mut().enumerate() {
                    if idx == self.tiers_attempted as usize {
                        account.is_primary = true;
                        account.is_fallback = false;
                    } else {
                        account.is_primary = false;
                        account.is_fallback = true;
                    }
                }

                self.inner = ExecutionPlanImpl::new(
                    ExecutionPlanType::Cascading,
                    self.inner.context().clone(),
                    planned_accounts,
                )
                .with_max_retries(1)
                .with_timeout(self.quality_config.per_tier_timeout_ms as u32 / 1000);
            }

            true
        } else {
            false
        }
    }

    /// Adds to the total cost accumulated.
    ///
    /// # Arguments
    ///
    /// * `cost_microdollars` - Cost to add in microdollars
    pub fn add_cost(&mut self, cost_microdollars: u64) {
        self.total_cost_microdollars = self
            .total_cost_microdollars
            .saturating_add(cost_microdollars);
    }

    /// Returns the total cost in microdollars.
    pub fn total_cost_microdollars(&self) -> u64 {
        self.total_cost_microdollars
    }

    /// Returns the number of tiers attempted so far.
    pub fn tiers_attempted(&self) -> u32 {
        self.tiers_attempted
    }

    /// Returns the total number of tiers available.
    pub fn total_tiers(&self) -> usize {
        self.tiers.len()
    }

    /// Returns whether we have exhausted all tiers.
    pub fn is_exhausted(&self) -> bool {
        (self.tiers_attempted as usize) >= self.tiers.len()
    }

    /// Executes all tiers in order with cascading logic.
    ///
    /// # Arguments
    ///
    /// * `config` - Execution configuration
    /// * `response_text` - Response text to evaluate for quality
    /// * `tokens_used` - Input and output tokens used in the request
    ///
    /// # Returns
    ///
    /// An `ExecutionResult` containing the outcome and cost information
    ///
    /// ## Execution Logic
    ///
    /// 1. If streaming is enabled, execute only the first tier and return
    /// 2. If quality escalation is disabled, execute all tiers until success or exhaustion
    /// 3. If quality escalation is enabled:
    ///    - Start with cheapest tier
    ///    - If quality >= acceptable, stop and return result
    ///    - If quality < acceptable, escalate to next tier
    ///    - Continue until success or exhaustion
    pub fn execute(
        &mut self,
        config: ExecutionConfig,
        response_text: &str,
        tokens_used: (u64, u64),
    ) -> ExecutionResult {
        // Streaming guard: skip cascading in streaming mode
        if config.stream {
            if self.current_tier().is_some() {
                // Execute only the first tier in streaming mode
                let planned_accounts: Vec<PlannedAccount> = self
                    .tiers
                    .iter()
                    .take(1)
                    .map(|t| t.account.clone())
                    .collect();

                // Create the inner execution plan with the first tier
                let _inner = ExecutionPlanImpl::new(
                    ExecutionPlanType::Cascading,
                    self.inner.context().clone(),
                    planned_accounts,
                );

                // Simulate execution (in real implementation, this would execute the actual request)
                // For now, we just record the cost
                let cost_estimate = 1000; // Simulated cost for demonstration
                self.add_cost(cost_estimate);

                return ExecutionResult::success(
                    0,
                    response_text.to_string(),
                    tokens_used,
                    cost_estimate,
                    self.total_cost_microdollars(),
                    None, // No quality evaluation in streaming mode
                    false,
                );
            }
        }

        // Non-streaming execution with cascading logic
        let mut used_quality_escalation = false;

        // Loop through tiers
        while let Some(tier) = self.current_tier() {
            // Extract tier data needed after mutable operations
            let tier_order = tier.tier_order;
            let tier_accounts: Vec<PlannedAccount> =
                self.tiers.iter().map(|t| t.account.clone()).collect();

            // Create the inner execution plan for this tier
            let _inner = ExecutionPlanImpl::new(
                ExecutionPlanType::Cascading,
                self.inner.context().clone(),
                tier_accounts,
            );

            // Simulate execution (in real implementation, this would execute the actual request)
            // For now, we record the cost and use the provided response
            // A real implementation would execute inner.execute() with actual LLM call
            let cost_estimate = 1000; // Simulated cost for demonstration

            // Check if we have exceeded cost budget
            if config.max_cost_microdollars > 0
                && self.total_cost_microdollars() + cost_estimate > config.max_cost_microdollars
            {
                // Stop execution if we exceed budget
                break;
            }

            self.add_cost(cost_estimate);

            // Evaluate quality if enabled and we have a response
            let final_quality_score = if config.enable_quality_escalation
                && tier_order > 0 // Only evaluate tiers after the first
                && !response_text.trim().is_empty()
            {
                used_quality_escalation = true;

                // Create quality evaluation span for tracing
                let quality_span = QualityEvaluationSpan::new(
                    &self.inner.context().request_id,
                    &tier.account.account_id,
                    tier_order,
                );

                // Actually evaluate the response quality using block_on since execute() is sync
                let quality_score =
                    futures::executor::block_on(self.quality_gate.evaluate_quality(
                        &tier.account,
                        response_text,
                        &tier.account.health,
                    ));

                quality_span.finish(
                    quality_score.score,
                    quality_score.is_acceptable,
                    &quality_score.checks_failed,
                );

                Some(quality_score.score)
            } else {
                None
            };

            // for a tier, but in cascading, we always return success after exhausting tiers
            let quality_acceptable = match final_quality_score {
                Some(score) => score >= self.quality_config.min_quality_score,
                None => true,
            };

            // If quality is acceptable, return success
            if quality_acceptable {
                return ExecutionResult::success(
                    tier_order as usize,
                    response_text.to_string(),
                    tokens_used,
                    cost_estimate,
                    self.total_cost_microdollars(),
                    final_quality_score,
                    used_quality_escalation,
                );
            }

            // If quality is not acceptable, escalate to next tier
            if !self.is_exhausted() && self.escalate_to_next_tier() {
                continue;
            } else {
                break;
            }
        }

        // All tiers exhausted or failed
        ExecutionResult::failure()
    }
}

impl ExecutionPlan for CascadingExecutionPlan {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::app::services::execution_plan::ExecutionContext;
    use crate::app::services::quality::evaluator::{HeuristicQualityEvaluator, QualityConfig};
    use crate::domain::entities::{Account, AccountHealth, Provider};

    fn create_test_provider_pricing() -> Vec<ProviderPricing> {
        vec![
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
            ProviderPricing {
                provider_id: "groq".to_string(),
                input_price_per_1m: 1.0,
                output_price_per_1m: 2.0,
            },
        ]
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
            (
                Account::new_api_key("acc-3", "groq", "sk-test-3"),
                Provider::new("groq", "Groq", "https://api.groq.com/openai/v1"),
                AccountHealth::new("acc-3"),
            ),
        ]
    }

    fn create_test_context() -> ExecutionContext {
        ExecutionContext::new("test-req-1", "gpt-4")
    }

    #[tokio::test]
    async fn test_cascading_tier_new() {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health = AccountHealth::new("test-acc");
        let account = PlannedAccount::new("test-acc", &provider, health);

        let tier = CascadingTier::new(account, "gpt-4", 0);

        assert_eq!(tier.account.account_id, "test-acc");
        assert_eq!(tier.model_id, "gpt-4");
        assert_eq!(tier.tier_order, 0);
    }

    #[tokio::test]
    async fn test_cascading_execution_plan_new() {
        let context = create_test_context();
        let accounts = create_test_accounts();
        let pricing = create_test_provider_pricing();
        let model_ids = vec![
            "gpt-4".to_string(),
            "claude-2".to_string(),
            "mixtral".to_string(),
        ];
        let quality_config = QualityConfig::default();
        let quality_gate = Arc::new(HeuristicQualityEvaluator::new());

        let plan = CascadingExecutionPlan::new(
            context,
            accounts,
            pricing,
            model_ids,
            quality_config,
            quality_gate,
        );

        assert_eq!(plan.plan_type(), ExecutionPlanType::Cascading);
        assert_eq!(plan.tiers.len(), 3);
        assert_eq!(plan.total_tiers(), 3);
        assert_eq!(plan.tiers_attempted(), 0);
        assert!(!plan.is_exhausted());

        // Check that tiers are ordered by cost (groq should be first as cheapest)
        assert_eq!(plan.tiers[0].account.provider_id, "groq");
        assert_eq!(plan.tiers[1].account.provider_id, "openai");
        assert_eq!(plan.tiers[2].account.provider_id, "anthropic");
    }

    #[tokio::test]
    async fn test_cascading_current_tier() {
        let context = create_test_context();
        let accounts = create_test_accounts();
        let pricing = create_test_provider_pricing();
        let model_ids = vec![
            "gpt-4".to_string(),
            "claude-2".to_string(),
            "mixtral".to_string(),
        ];
        let quality_config = QualityConfig::default();
        let quality_gate = Arc::new(HeuristicQualityEvaluator::new());

        let mut plan = CascadingExecutionPlan::new(
            context,
            accounts,
            pricing,
            model_ids,
            quality_config,
            quality_gate,
        );

        // Initially should be on first tier
        assert!(plan.current_tier().is_some());
        assert_eq!(plan.current_tier().unwrap().tier_order, 0);
        assert_eq!(plan.tiers_attempted(), 0);

        // After escalating to next tier
        plan.escalate_to_next_tier();
        assert!(plan.current_tier().is_some());
        assert_eq!(plan.current_tier().unwrap().tier_order, 1);
        assert_eq!(plan.tiers_attempted(), 1);

        // After escalating to final tier
        plan.escalate_to_next_tier();
        assert_eq!(plan.tiers_attempted(), 2);
        assert!(plan.current_tier().is_some());
        assert_eq!(plan.current_tier().unwrap().tier_order, 2);
        assert!(!plan.is_exhausted());

        // After escalating to exhausted state
        plan.escalate_to_next_tier();
        assert_eq!(plan.tiers_attempted(), 3);
        assert!(plan.is_exhausted());

        // Trying to escalate further should return false
        let result = plan.escalate_to_next_tier();
        assert!(!result);
        assert_eq!(plan.tiers_attempted(), 3);
        assert!(plan.is_exhausted());
    }

    #[tokio::test]
    async fn test_cascading_cost_tracking() {
        let context = create_test_context();
        let accounts = create_test_accounts();
        let pricing = create_test_provider_pricing();
        let model_ids = vec![
            "gpt-4".to_string(),
            "claude-2".to_string(),
            "mixtral".to_string(),
        ];
        let quality_config = QualityConfig::default();
        let quality_gate = Arc::new(HeuristicQualityEvaluator::new());

        let mut plan = CascadingExecutionPlan::new(
            context,
            accounts,
            pricing,
            model_ids,
            quality_config,
            quality_gate,
        );

        assert_eq!(plan.total_cost_microdollars(), 0);

        plan.add_cost(1500);
        assert_eq!(plan.total_cost_microdollars(), 1500);

        plan.add_cost(3500);
        assert_eq!(plan.total_cost_microdollars(), 5000);
    }
}
