//! Common test utilities for integration tests
//!
//! This module provides utilities for testing the execution plan module,
//! failover system, and handler integration tests.

#![allow(dead_code)]

pub mod containers;
pub mod errors;

use async_trait::async_trait;
use mockall::predicate::eq;
use rust_llm_api_router::app::services::account_rotation::AccountSelector;
use rust_llm_api_router::app::services::execution_plan::{
    ExecutionContext, ExecutionPlanner, ExecutionPlannerConfig, PlanningOptions,
};
use rust_llm_api_router::app::services::failover::FailoverManager;
use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::entities::Account;
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::DomainError;
use rust_llm_api_router::infrastructure::gateway::llm_gateway::{
    default_providers, ProviderConfig,
};
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository};
use rust_llm_api_router::presentation::state::AppState;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// MOCK REPOSITORY BUILDERS
// ============================================================================

mockall::mock! {
    /// Mock repository for testing with configurable behavior
    pub TestAccountRepo {}

    #[async_trait]
    impl AccountRepository for TestAccountRepo {
        async fn save(&self, account: Account) -> Result<Account, DomainError>;
        async fn find_all(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_by_id(&self, id: &str) -> Result<Account, DomainError>;
        async fn find_active(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_active_by_provider(&self, provider_id: &str) -> Result<Vec<Account>, DomainError>;
        async fn delete(&self, id: &str) -> Result<(), DomainError>;
    }
}

impl Clone for MockTestAccountRepo {
    fn clone(&self) -> Self {
        MockTestAccountRepo::new()
    }
}

// ============================================================================
// REPOSITORY SETUP HELPERS
// ============================================================================

/// Create a repository with a single account
pub async fn create_repo_with_account(provider_id: &str) -> Arc<dyn AccountRepository> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::new().expect("Should create repository"));

    let account = Account::new(
        format!("{}-account-1", provider_id),
        provider_id,
        "sk-test-key",
    );
    repo.save(account).await.expect("Should save account");

    repo
}

/// Create a repository with multiple accounts
pub async fn create_repo_with_accounts(
    provider_id: &str,
    count: usize,
) -> Arc<dyn AccountRepository> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::new().expect("Should create repository"));

    for i in 0..count {
        let account = Account::new(
            format!("{}-account-{}", provider_id, i + 1),
            provider_id,
            format!("sk-test-key-{}", i + 1),
        );
        repo.save(account).await.expect("Should save account");
    }

    repo
}

/// Create a repository with multiple providers
pub async fn create_repo_with_multi_provider() -> Arc<dyn AccountRepository> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::new().expect("Should create repository"));

    // OpenAI accounts
    for i in 0..3 {
        let account = Account::new(
            format!("openai-account-{}", i + 1),
            "openai",
            format!("sk-openai-key-{}", i + 1),
        );
        repo.save(account).await.expect("Should save account");
    }

    // Anthropic accounts
    let account = Account::new("anthropic-account-1", "anthropic", "sk-anthropic-key");
    repo.save(account).await.expect("Should save account");

    // Groq accounts
    let account = Account::new("groq-account-1", "groq", "sk-groq-key");
    repo.save(account).await.expect("Should save account");

    repo
}

// ============================================================================
// FAILOVER MANAGER HELPERS
// ============================================================================

/// Create a failover manager with a test account
pub async fn create_manager_with_provider(provider_id: &str, _endpoint: &str) -> FailoverManager {
    let repo = create_repo_with_account(provider_id).await;
    FailoverManager::with_round_robin(repo)
}

/// Create a failover manager with custom selector
pub async fn create_manager_with_selector(selector: AccountSelector) -> FailoverManager {
    let repo = Arc::new(JsonAccountRepository::new().expect("Should create repository"));
    FailoverManager::new(repo, selector, 3)
}

/// Create a failover manager with multiple accounts
pub async fn create_manager_with_multi_account(provider_id: &str, count: usize) -> FailoverManager {
    let repo = create_repo_with_accounts(provider_id, count).await;
    FailoverManager::with_round_robin(repo)
}

/// Create a failover manager with retry configuration
pub async fn create_manager_with_retries(provider_id: &str, retries: u32) -> FailoverManager {
    let repo = create_repo_with_account(provider_id).await;
    FailoverManager::new(repo, AccountSelector::round_robin(), retries)
}

// ============================================================================
// EXECUTION PLANNER HELPERS
// ============================================================================

/// Create an execution planner with default config
pub async fn create_planner() -> ExecutionPlanner<MockTestAccountRepo> {
    let mock_repo = MockTestAccountRepo::new();
    ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default())
}

/// Create an execution planner with reliability config
pub async fn create_reliability_planner() -> ExecutionPlanner<MockTestAccountRepo> {
    let mock_repo = MockTestAccountRepo::new();
    ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::reliability())
}

/// Create an execution planner with mock repository setup
pub fn create_planner_with_mock<F>(setup: F) -> ExecutionPlanner<MockTestAccountRepo>
where
    F: FnOnce(&mut MockTestAccountRepo),
{
    let mut mock_repo = MockTestAccountRepo::new();
    setup(&mut mock_repo);
    ExecutionPlanner::new(Arc::new(mock_repo), ExecutionPlannerConfig::default())
}

/// Setup mock to return accounts for a provider
pub fn setup_mock_provider_accounts(
    mock: &mut MockTestAccountRepo,
    provider_id: &str,
    accounts: Vec<Account>,
) {
    mock.expect_find_active_by_provider()
        .with(eq(provider_id.to_string()))
        .returning(move |_| Ok(accounts.clone()));
}

// ============================================================================
// CONTEXT HELPERS
// ============================================================================

/// Create a basic execution context
pub fn create_basic_context(model: &str) -> ExecutionContext {
    ExecutionContext::new("test-request", model)
}

/// Create context with provider preference
pub fn create_context_with_provider(model: &str, provider: &str) -> ExecutionContext {
    ExecutionContext::new("test-request", model)
        .with_preferred_providers(vec![provider.to_string()])
}

/// Create context with reliability options
pub fn create_reliability_context(model: &str) -> ExecutionContext {
    ExecutionContext::new("test-request", model)
        .with_planning_options(PlanningOptions::reliability())
}

/// Create context with cost optimized options
pub fn create_cost_optimized_context(model: &str) -> ExecutionContext {
    ExecutionContext::new("test-request", model)
        .with_planning_options(PlanningOptions::cost_optimized())
}

/// Create context with low latency options
pub fn create_low_latency_context(model: &str) -> ExecutionContext {
    ExecutionContext::new("test-request", model)
        .with_planning_options(PlanningOptions::low_latency())
}

// ============================================================================
// APP STATE HELPERS
// ============================================================================

/// Create AppState for testing with default providers
pub fn create_test_app_state(
    account_repo: Arc<dyn AccountRepository>,
    http_client: Arc<HttpClient>,
) -> AppState {
    let provider_config = default_providers();
    let settings = Settings::default();

    AppState::with_provider_config(settings, http_client, account_repo, provider_config).unwrap()
}

/// Create AppState with custom provider config (for mock servers)
pub fn create_test_app_state_with_config(
    account_repo: Arc<dyn AccountRepository>,
    http_client: Arc<HttpClient>,
    provider_config: HashMap<String, ProviderConfig>,
) -> AppState {
    let settings = Settings::default();

    AppState::with_provider_config(settings, http_client, account_repo, provider_config).unwrap()
}
