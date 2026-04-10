//! E2E Tests for Cascading Routing (Issue #24)
//!
//! Comprehensive tests covering auto-selection, quality module, CascadingTier,
//! streaming guard, and edge cases.
//!
//! Follows rust-skills guidelines: arrange/act/assert, descriptive names, #[tokio::test].

use rust_llm_api_router::app::services::execution_plan::types::PlannedAccount;
use rust_llm_api_router::app::services::execution_plan::{
    CascadingExecutionPlan, CascadingTier, ExecutionConfig, ExecutionContext, PlanningOptions,
    ProviderPricing,
};
use rust_llm_api_router::app::services::quality::evaluator::{
    HeuristicQualityEvaluator, QualityConfig, QualityGate,
};
use rust_llm_api_router::domain::entities::{Account, AccountHealth, Provider};
use std::sync::Arc;

// ============================================================================
// HELPER FUNCTIONS (mirroring cascading.rs unit test helpers)
// ============================================================================

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

fn create_mock_planned_account() -> PlannedAccount {
    let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
    let health = AccountHealth::new("test-acc");
    PlannedAccount::new("test-acc", &provider, health)
}

// ============================================================================
// 1. AUTO-SELECTION TESTS
// ============================================================================

/// Test: PlanningOptions::default() should have budget_mode = false
#[test]
fn test_planning_options_budget_mode_default_false() {
    // Arrange
    let options = PlanningOptions::default();

    // Act & Assert
    assert!(!options.budget_mode, "Default budget_mode should be false");
    assert!(!options.enable_cascading, "Default enable_cascading should be false");
}

/// Test: PlanningOptions::cost_optimized() should enable budget_mode
#[test]
fn test_planning_options_cost_optimized_enables_budget_mode() {
    // Arrange
    let options = PlanningOptions::cost_optimized();

    // Act & Assert
    assert!(options.budget_mode, "Cost optimized should enable budget_mode");
    assert!(options.cost_optimized, "Cost optimized should set cost_optimized flag");
    assert!(
        !options.enable_cascading,
        "Cost optimized should not enable cascading by default"
    );
}

/// Test: PlanningOptions::cascading() preset
#[test]
fn test_planning_options_cascading_preset() {
    // Arrange
    let options = PlanningOptions::cascading();

    // Act & Assert
    assert!(options.enable_cascading, "Cascading preset should enable cascading");
    assert!(options.budget_mode, "Cascading preset should enable budget_mode");
    assert!(options.cost_optimized, "Cascading preset should enable cost_optimized");
    assert!(!options.enable_failover, "Cascading preset should disable failover");
    assert!(!options.enable_load_balancing, "Cascading preset should disable load balancing");
    assert_eq!(options.max_retries, 1, "Cascading preset should have max_retries = 1");
}

/// Test: budget_mode triggers cascading selection in execution context
#[test]
fn test_budget_mode_triggers_cascading_selection() {
    // Arrange
    let context =
        ExecutionContext::new("req-1", "gpt-4").with_planning_options(PlanningOptions::cascading());

    // Act & Assert
    assert!(context.planning_options.budget_mode, "Budget mode should be true");
    assert!(context.planning_options.enable_cascading, "Cascading should be enabled");
    assert!(context.planning_options.cost_optimized, "Cost optimized should be enabled");
}

// ============================================================================
// 2. QUALITY MODULE TESTS
// ============================================================================

/// Test: QualityConfig default values
#[test]
fn test_quality_config_defaults() {
    // Arrange
    let config = QualityConfig::default();

    // Act & Assert
    assert_eq!(config.min_quality_score, 0.75, "Default min quality score should be 0.75");
    assert_eq!(config.min_response_length, 10, "Default min response length should be 10");
    assert_eq!(config.max_tiers, 3, "Default max tiers should be 3");
    assert_eq!(config.per_tier_timeout_ms, 5000, "Default per tier timeout should be 5000ms");
}

/// Test: HeuristicQualityEvaluator accepts good response (passes all checks)
#[tokio::test]
async fn test_heuristic_evaluator_accepts_good_response() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    let response = "This is a well-formed response that meets all quality criteria.";

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    assert!(score.is_acceptable, "Good response should be acceptable");
    assert_eq!(score.score, 1.0, "Good response should have perfect score");
    assert!(score.checks_failed.is_empty(), "No checks should fail for good response");
}

/// Test: HeuristicQualityEvaluator rejects incoherent and short response
/// Must fail 2+ checks to get score < 0.75
#[tokio::test]
async fn test_heuristic_evaluator_rejects_incoherent_and_short() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    // "I cannot" fails coherence ("i cannot") + length (8 chars < 10) = 2/4 = 0.5 < 0.75
    let response = "I cannot";

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    assert!(!score.is_acceptable, "Incoherent short response should be unacceptable");
    assert!(score.score < 0.75, "Score should be less than 0.75");
    assert!(score.checks_failed.contains(&"length".to_string()), "Length check should fail");
    assert!(
        score.checks_failed.contains(&"coherence".to_string()),
        "Coherence check should fail"
    );
    assert!(score.checks_failed.len() >= 2, "At least 2 checks should fail");
}

/// Test: HeuristicQualityEvaluator flags error patterns
#[tokio::test]
async fn test_heuristic_evaluator_flags_error_patterns() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    let response = "As an AI language model, I don't have personal opinions.";

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    // The response contains "as an AI" pattern, so coherence check should fail.
    assert!(
        score.checks_failed.contains(&"coherence".to_string()),
        "Coherence check should fail"
    );
    // However, the response passes other checks (length, completeness, structure) => 3/4 = 0.75 >= 0.75 => acceptable.
    assert!(score.is_acceptable, "Score of 0.75 should still be acceptable");
    assert_eq!(score.score, 0.75, "Score should be exactly 0.75");
    // The response may still pass other checks (length, completeness, structure)
}

/// Test: HeuristicQualityEvaluator accepts valid JSON-like response
#[tokio::test]
async fn test_heuristic_evaluator_accepts_valid_json() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    let response = r#"{"status": "success", "data": {"id": 1}}"#;

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    // Valid JSON should pass structure check
    assert!(
        !score.checks_failed.contains(&"structure".to_string()),
        "Structure check should pass"
    );
    // Other checks may fail? length > 10, completeness ends with '}' good, coherence no error patterns.
    // So should be acceptable.
    assert!(score.is_acceptable, "Valid JSON response should be acceptable");
}

/// Test: Quality threshold boundary - score exactly 0.75 (3 passes)
#[tokio::test]
async fn test_quality_threshold_boundary_exactly_075() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    // Craft a response that fails exactly one check (completeness) but passes length, structure, coherence.
    // Example: ends with a comma (fails completeness) but length >10, no JSON issues, no error patterns.
    let response = "This response is acceptable, but ends with a comma,";

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    assert!(score.is_acceptable, "Score of exactly 0.75 should be acceptable");
    assert_eq!(score.score, 0.75, "Score should be exactly 0.75");
    assert_eq!(score.checks_failed.len(), 1, "Exactly one check should fail");
    assert!(
        score.checks_failed.contains(&"completeness".to_string()),
        "Completeness check should fail"
    );
}

/// Test: Quality threshold boundary - score just below 0.75 (2 passes)
#[tokio::test]
async fn test_quality_threshold_boundary_below_075() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let account = create_mock_planned_account();
    let health = AccountHealth::new("test-acc");
    // Craft a response that fails exactly two checks (length + coherence)
    // "Hi, I cannot" -> length 11? Actually "Hi, I cannot" length 12? Wait need length <10 and coherence fail.
    // Let's use "I cannot" (8 chars) fails length and coherence = 2/4 = 0.5 <0.75.
    let response = "I cannot";

    // Act
    let score = evaluator
        .evaluate_quality(&account, response, &health)
        .await;

    // Assert
    assert!(!score.is_acceptable, "Score below 0.75 should be unacceptable");
    assert!(score.score < 0.75, "Score should be less than 0.75");
    assert_eq!(score.checks_failed.len(), 2, "Exactly two checks should fail");
}

// ============================================================================
// 3. CASCADINGTIER TESTS
// ============================================================================

/// Test: CascadingTier creation
#[test]
fn test_cascading_tier_creation() {
    // Arrange
    let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
    let health = AccountHealth::new("test-acc");
    let planned = PlannedAccount::new("test-acc", &provider, health);
    let model_id = "gpt-4";

    // Act
    let tier = CascadingTier::new(planned, model_id, 0);

    // Assert
    assert_eq!(tier.account.account_id, "test-acc");
    assert_eq!(tier.account.provider_id, "openai");
    assert_eq!(tier.model_id, "gpt-4");
    assert_eq!(tier.tier_order, 0);
}

/// Test: CascadingTier ordering (lower tier_order is tried first)
#[test]
fn test_cascading_tier_ordering() {
    // Arrange
    let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
    let health = AccountHealth::new("test-acc");
    let planned = PlannedAccount::new("test-acc", &provider, health);

    // Act
    let tier0 = CascadingTier::new(planned.clone(), "model-a", 0);
    let tier1 = CascadingTier::new(planned.clone(), "model-b", 1);
    let tier2 = CascadingTier::new(planned, "model-c", 2);

    // Assert
    assert!(tier0.tier_order < tier1.tier_order, "Tier 0 should be before tier 1");
    assert!(tier1.tier_order < tier2.tier_order, "Tier 1 should be before tier 2");
    assert_eq!(tier0.tier_order, 0);
    assert_eq!(tier1.tier_order, 1);
    assert_eq!(tier2.tier_order, 2);
}

// ============================================================================
// 4. STREAMING GUARD TESTS
// ============================================================================

/// Test: ExecutionConfig flags for streaming vs non-streaming
#[test]
fn test_streaming_config_flags() {
    // Arrange
    let streaming = ExecutionConfig::streaming();
    let non_streaming = ExecutionConfig::non_streaming();

    // Act & Assert
    assert!(streaming.stream, "Streaming config should have stream = true");
    assert!(
        !streaming.enable_quality_escalation,
        "Streaming should disable quality escalation"
    );
    assert!(!non_streaming.stream, "Non-streaming config should have stream = false");
    assert!(
        non_streaming.enable_quality_escalation,
        "Non-streaming should enable quality escalation"
    );
}

/// Test: Streaming prevents cascading concept (cascading is skipped)
/// This test creates a CascadingExecutionPlan and calls execute with stream = true.
/// It verifies that only the first tier is used, no quality escalation occurs,
/// and the result is successful.
#[tokio::test]
async fn test_streaming_prevents_cascading_concept() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let context = create_test_context();
    let accounts = create_test_accounts();
    let pricing = create_test_provider_pricing();
    let model_ids = vec![
        "gpt-4".to_string(),
        "claude-2".to_string(),
        "mixtral".to_string(),
    ];
    let quality_config = QualityConfig::default();
    let quality_gate = Arc::new(evaluator);

    let mut plan = CascadingExecutionPlan::new(
        context,
        accounts,
        pricing,
        model_ids,
        quality_config,
        quality_gate,
    );

    let config = ExecutionConfig::streaming();
    let response = "Streaming response that may be truncated";
    let tokens_used = (100, 50);

    // Act
    let result = plan.execute(config, response, tokens_used).await;

    // Assert
    assert!(result.success, "Execution should succeed in streaming mode");
    assert_eq!(result.final_tier_index, 0, "Should only use first tier");
    assert!(
        !result.used_quality_escalation,
        "Quality escalation should not be used in streaming"
    );
    assert!(result.final_quality_score.is_none(), "No quality score in streaming mode");
}

// ============================================================================
// 5. EDGE CASES
// ============================================================================

/// Test: Cost budget zero means unlimited (cascading execution not stopped by budget)
#[tokio::test]
async fn test_cost_budget_zero_means_unlimited() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let context = create_test_context();
    let accounts = create_test_accounts();
    let pricing = create_test_provider_pricing();
    let model_ids = vec![
        "gpt-4".to_string(),
        "claude-2".to_string(),
        "mixtral".to_string(),
    ];
    let quality_config = QualityConfig::default();
    let quality_gate = Arc::new(evaluator);

    let mut plan = CascadingExecutionPlan::new(
        context,
        accounts,
        pricing,
        model_ids,
        quality_config,
        quality_gate,
    );

    // Set max_cost_microdollars = 0 (unlimited)
    let config = ExecutionConfig::default().with_cost_budget(0);
    let response = "Good response that passes quality checks";
    let tokens_used = (100, 50);

    // Act
    let result = plan.execute(config, response, tokens_used).await;

    // Assert
    // With unlimited budget, execution should succeed and not be stopped by budget.
    assert!(result.success, "Execution should succeed with unlimited budget");
    // Total cost should be positive (simulated cost per tier)
    assert!(result.total_cost_microdollars > 0, "Total cost should be positive");
}

/// Test: Cascading execution with cost budget enforcement
#[tokio::test]
async fn test_cost_budget_enforcement() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let context = create_test_context();
    let accounts = create_test_accounts();
    let pricing = create_test_provider_pricing();
    let model_ids = vec![
        "gpt-4".to_string(),
        "claude-2".to_string(),
        "mixtral".to_string(),
    ];
    let quality_config = QualityConfig::default();
    let quality_gate = Arc::new(evaluator);

    let mut plan = CascadingExecutionPlan::new(
        context,
        accounts,
        pricing,
        model_ids,
        quality_config,
        quality_gate,
    );

    // Set a very small budget (less than simulated cost per tier)
    // Simulated cost per tier is 1000 microdollars (see cascading.rs)
    let config = ExecutionConfig::default().with_cost_budget(500); // less than 1000
    let response = "Response";
    let tokens_used = (10, 5);

    // Act
    let result = plan.execute(config, response, tokens_used).await;

    // Assert
    // Should fail because budget exceeded after first tier? Actually the logic checks if total_cost + cost_estimate > max_cost.
    // Since max_cost = 500, cost_estimate = 1000, total_cost initially 0, so condition triggers break.
    // The loop will break and return ExecutionResult::failure().
    assert!(!result.success, "Execution should fail due to budget limit");
}

/// Test: Cascading execution with quality escalation (non-streaming)
#[tokio::test]
async fn test_cascading_execution_quality_escalation() {
    // Arrange
    let evaluator = HeuristicQualityEvaluator::new();
    let context = create_test_context();
    let accounts = create_test_accounts();
    let pricing = create_test_provider_pricing();
    let model_ids = vec![
        "gpt-4".to_string(),
        "claude-2".to_string(),
        "mixtral".to_string(),
    ];
    let quality_config = QualityConfig::default();
    let quality_gate = Arc::new(evaluator);

    let mut plan = CascadingExecutionPlan::new(
        context,
        accounts,
        pricing,
        model_ids,
        quality_config,
        quality_gate,
    );

    // Escalate to second tier (tier_order = 1) to trigger quality escalation
    // (quality escalation only applies to tiers after the first)
    assert!(plan.escalate_to_next_tier(), "Should escalate to second tier");

    // Non-streaming config with quality escalation enabled
    let config = ExecutionConfig::non_streaming();
    let response = "Good response";
    let tokens_used = (100, 50);

    // Act
    let result = plan.execute(config, response, tokens_used).await;

    // Assert
    assert!(result.success, "Execution should succeed");
    // Now quality escalation should be used because tier_order > 0
    assert!(result.used_quality_escalation, "Quality escalation should be used");
    assert!(result.final_quality_score.is_some(), "Quality score should be present");
    // Since quality simulated as 0.85 >= 0.75, execution succeeds at tier 1
    assert_eq!(result.final_tier_index, 1, "Should succeed on second tier");
}

// ============================================================================
// ADDITIONAL VALIDATION TESTS (from original file)
// ============================================================================

/// Test: Quality config custom values
#[test]
fn test_quality_config_custom_values() {
    // Arrange
    let config = QualityConfig {
        min_quality_score: 0.9,
        min_response_length: 100,
        max_tiers: 5,
        per_tier_timeout_ms: 10000,
    };

    // Act & Assert
    assert_eq!(config.min_quality_score, 0.9);
    assert_eq!(config.min_response_length, 100);
    assert_eq!(config.max_tiers, 5);
    assert_eq!(config.per_tier_timeout_ms, 10000);
}

/// Test: Max tiers reasonable range
#[test]
fn test_max_tiers_reasonable_range() {
    // Arrange
    let config = QualityConfig::default();

    // Act & Assert
    assert!(config.max_tiers >= 2, "Max tiers should be at least 2");
    assert!(config.max_tiers <= 10, "Max tiers should be at most 10");
}

/// Test: Timeout reasonable range
#[test]
fn test_timeout_reasonable_range() {
    // Arrange
    let config = QualityConfig::default();

    // Act & Assert
    assert!(config.per_tier_timeout_ms >= 1000, "Timeout should be at least 1 second");
    assert!(config.per_tier_timeout_ms <= 30000, "Timeout should be at most 30 seconds");
}

/// Test: Cost budget positive enforces limit
#[test]
fn test_cost_budget_positive_enforces_limit() {
    // Arrange
    let budget = 10_000; // $0.01
    let config = ExecutionConfig::default().with_cost_budget(budget);

    // Act & Assert
    assert_eq!(config.max_cost_microdollars, budget);
}
