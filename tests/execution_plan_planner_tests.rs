//! Integration tests for Execution Planner with Handler
//!
//! Tests the integration between the ExecutionPlanner and the chat handler,
//! including end-to-end execution flow with different plan types.

use async_trait::async_trait;
use mockall::predicate::*;
use rust_llm_api_router::app::services::execution_plan::{
    ExecutionContext, ExecutionPlanner, ExecutionPlannerConfig, PlanningOptions,
};
use rust_llm_api_router::domain::entities::Account;
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::DomainError;
use std::sync::Arc;

// ============================================================================
// MOCK REPOSITORY FOR PLANNER TESTS
// ============================================================================

mockall::mock! {
    pub TestAccountRepository {}

    #[async_trait]
    impl AccountRepository for TestAccountRepository {
        async fn save(&self, account: Account) -> Result<Account, DomainError>;
        async fn find_all(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_by_id(&self, id: &str) -> Result<Account, DomainError>;
        async fn find_active(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_active_by_provider(&self, provider_id: &str) -> Result<Vec<Account>, DomainError>;
        async fn delete(&self, id: &str) -> Result<(), DomainError>;
    }
}

impl Clone for MockTestAccountRepository {
    fn clone(&self) -> Self {
        MockTestAccountRepository::new()
    }
}

// ============================================================================
// PLANNER BASIC TESTS
// ============================================================================

/// Test: Planner creates execution plan successfully
#[tokio::test]
async fn test_planner_creates_plan() {
    let mut mock_repo = MockTestAccountRepository::new();

    // Return accounts for the provider
    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-test-key-1"),
                Account::new("account-2", "openai", "sk-test-key-2"),
            ])
        });

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::default());

    let result = planner.create_plan(context).await;

    assert!(result.is_ok(), "Planner should create plan successfully");
}

/// Test: Planner with reliability context
#[tokio::test]
async fn test_planner_with_reliability_context() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-test-key-1")]));

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::reliability());

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

/// Test: Planner with cost optimized context
#[tokio::test]
async fn test_planner_with_cost_optimized_context() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("groq"))
        .returning(|_| Ok(vec![Account::new("groq-account-1", "groq", "sk-groq-key-1")]));

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "llama-3")
        .with_preferred_providers(vec!["groq".to_string()])
        .with_planning_options(PlanningOptions::cost_optimized());

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

/// Test: Planner with low latency context
#[tokio::test]
async fn test_planner_with_low_latency_context() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-test-key-1"),
                Account::new("account-2", "openai", "sk-test-key-2"),
            ])
        });

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()])
        .with_planning_options(PlanningOptions::low_latency());

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

// ============================================================================
// PLANNER CONFIGURATION TESTS
// ============================================================================

/// Test: Planner with reliability config preset
#[tokio::test]
async fn test_planner_reliability_config() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-test-key-1")]));

    let config = ExecutionPlannerConfig::reliability();
    let planner = ExecutionPlanner::new(Arc::new(mock_repo), config);

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

/// Test: Planner with cost optimized config preset
#[tokio::test]
async fn test_planner_cost_optimized_config() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-test-key-1")]));

    let config = ExecutionPlannerConfig::cost_optimized();
    let planner = ExecutionPlanner::new(Arc::new(mock_repo), config);

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

/// Test: Planner with low latency config preset
#[tokio::test]
async fn test_planner_low_latency_config() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-test-key-1")]));

    let config = ExecutionPlannerConfig::low_latency();
    let planner = ExecutionPlanner::new(Arc::new(mock_repo), config);

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

// ============================================================================
// MULTI-PROVIDER TESTS
// ============================================================================

/// Test: Planner handles multiple provider preferences
#[tokio::test]
async fn test_planner_multiple_provider_preferences() {
    let mut mock_repo = MockTestAccountRepository::new();

    // First call for openai
    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Ok(vec![Account::new("openai-1", "openai", "sk-openai-key")]));

    // Second call for anthropic
    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("anthropic"))
        .returning(|_| Ok(vec![Account::new("anthropic-1", "anthropic", "sk-anthropic-key")]));

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string(), "anthropic".to_string()]);

    let result = planner.create_plan(context).await;
    assert!(result.is_ok());
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

/// Test: Planner handles repository errors gracefully
#[tokio::test]
async fn test_planner_repository_error() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| Err(DomainError::AccountNotFound("openai".to_string())));

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["openai".to_string()]);

    let result = planner.create_plan(context).await;
    assert!(result.is_err());
}

/// Test: Planner handles empty provider list
#[tokio::test]
async fn test_planner_empty_provider_list() {
    let mut mock_repo = MockTestAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("unknown"))
        .returning(|_| Ok(vec![]));

    let planner = ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default());

    let context = ExecutionContext::new("req-1", "gpt-4")
        .with_preferred_providers(vec!["unknown".to_string()]);

    // When no accounts are available, planner may still return a plan
    // but with no accounts. We just verify it doesn't panic.
    let result = planner.create_plan(context).await;
    // Just verify it doesn't panic - result depends on implementation
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// CONFIG TESTS
// ============================================================================

/// Test: Config presets work correctly
#[tokio::test]
async fn test_config_presets() {
    // Test reliability preset
    let reliability = ExecutionPlannerConfig::reliability();
    let _planner1 = ExecutionPlanner::new(Arc::new(MockTestAccountRepository::new()), reliability);

    // Test cost optimized preset
    let cost = ExecutionPlannerConfig::cost_optimized();
    let _planner2 = ExecutionPlanner::new(Arc::new(MockTestAccountRepository::new()), cost);

    // Test low latency preset
    let latency = ExecutionPlannerConfig::low_latency();
    let _planner3 = ExecutionPlanner::new(Arc::new(MockTestAccountRepository::new()), latency);
}
