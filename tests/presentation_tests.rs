//! Integration tests for presentation layer (state management)
//!
//! These tests verify AppState creation and management.

use std::sync::Arc;
use tempfile::TempDir;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository, Metrics};
use rust_llm_api_router::presentation::AppState;

// ============================================================================
// AppState Tests
// ============================================================================

#[test]
fn test_app_state_creation() {
    let _temp_dir = TempDir::new().unwrap();

    let settings = Settings::default();

    let state = AppState::new(settings).unwrap();

    // Verify state created successfully
    assert_eq!(state.config.app_port, 8080);
    assert_eq!(state.config.log_level, "info");
}

#[test]
fn test_app_state_clone() {
    let temp_dir = TempDir::new().unwrap();

    let settings = Settings::default();

    let http_client = Arc::new(HttpClient::new().unwrap());
    let account_repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    let provider_config = default_providers();

    let state = AppState::with_provider_config(settings, http_client, account_repo, provider_config)
        .unwrap();

    // Clone state (required for Axum multi-request handling)
    let state_clone = state.clone();

    // Verify clone has same configuration
    assert_eq!(state_clone.config.app_port, state.config.app_port);
    assert_eq!(state_clone.config.log_level, state.config.log_level);
}

#[tokio::test]
async fn test_app_state_with_accounts() {
    let temp_dir = TempDir::new().unwrap();

    let settings = Settings::default();

    let http_client = Arc::new(HttpClient::new().unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let account_repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    // Add accounts first
    account_repo
        .save(Account::new("acc-1", "openai", "sk-key-1"))
        .await
        .unwrap();
    account_repo
        .save(Account::new("acc-2", "groq", "sk-key-2"))
        .await
        .unwrap();

    let provider_config = default_providers();
    let state = AppState::with_provider_config(settings, http_client, account_repo.clone(), provider_config)
        .unwrap();

    // Verify accounts are accessible
    let accounts = state.account_repo.find_all().await.unwrap();
    assert_eq!(accounts.len(), 2);
}

#[tokio::test]
async fn test_state_account_operations() {
    let temp_dir = TempDir::new().unwrap();

    let settings = Settings::default();

    let http_client = Arc::new(HttpClient::new().unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let account_repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    let provider_config = default_providers();
    let state = AppState::with_provider_config(settings, http_client, account_repo.clone(), provider_config)
        .unwrap();

    // Test repository operations through state
    // Add account
    let account = Account::new("test-state", "openai", "sk-test");
    account_repo.save(account).await.unwrap();

    // Find through state
    let found = state.account_repo.find_by_id("test-state").await.unwrap();
    assert_eq!(found.provider_id, "openai");

    // Delete through state
    state.account_repo.delete("test-state").await.unwrap();

    // Verify deleted
    let result = state.account_repo.find_by_id("test-state").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_state_metrics_accessible() {
    let temp_dir = TempDir::new().unwrap();

    let settings = Settings::default();

    let http_client = Arc::new(HttpClient::new().unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let account_repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    let provider_config = default_providers();
    let state = AppState::with_provider_config(settings, http_client, account_repo, provider_config)
        .unwrap();

    // Verify metrics registry is accessible
    let _registry = state.metrics.registry.gather();
}

#[tokio::test]
async fn test_state_http_client_accessible() {
    let temp_dir = TempDir::new().unwrap();

    let http_client = Arc::new(HttpClient::new().unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let account_repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    let provider_config = default_providers();
    let state = AppState::with_provider_config(
        Settings::default(),
        http_client.clone(),
        account_repo,
        provider_config,
    )
    .unwrap();

    // Verify http_client is accessible
    let _client = state.http_client.client();
}
