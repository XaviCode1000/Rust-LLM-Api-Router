//! Common test utilities for integration tests

#![allow(dead_code)]

pub mod containers;

use rust_llm_api_router::app::services::{AccountSelector, FailoverManager};
use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::{
    default_providers, ProviderConfig,
};
use rust_llm_api_router::infrastructure::{
    HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics,
};
use rust_llm_api_router::presentation::state::AppState;
use std::collections::HashMap;
use std::sync::Arc;

/// Create a failover manager with a test account pointing to a mock endpoint
pub async fn create_manager_with_provider(provider_id: &str, _endpoint: &str) -> FailoverManager {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::new().expect("Should create repository"));

    let account = Account::new(
        format!("{}-test-1", provider_id),
        provider_id,
        "sk-test-key",
    );
    repo.save(account).await.expect("Should save account");

    FailoverManager::with_round_robin(repo)
}

/// Helper to create a manager with custom selector
pub async fn create_manager_with_selector(selector: AccountSelector) -> FailoverManager {
    let repo = Arc::new(JsonAccountRepository::new().expect("Should create repository"));

    FailoverManager::new(repo, selector, 3)
}

/// Create AppState for testing with default providers
pub fn create_test_app_state(
    account_repo: Arc<dyn AccountRepository>,
    http_client: Arc<HttpClient>,
) -> AppState {
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        account_repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();

    AppState {
        config: settings,
        http_client,
        metrics,
        account_repo,
        llm_gateway,
        provider_config,
    }
}

/// Create AppState with custom provider config (for mock servers)
pub fn create_test_app_state_with_config(
    account_repo: Arc<dyn AccountRepository>,
    http_client: Arc<HttpClient>,
    provider_config: HashMap<String, ProviderConfig>,
) -> AppState {
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config_arc = Arc::new(provider_config.clone());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        account_repo.clone(),
        provider_config,
        3600,
    ));
    let settings = Settings::default();

    AppState {
        config: settings,
        http_client,
        metrics,
        account_repo,
        llm_gateway,
        provider_config: provider_config_arc,
    }
}
