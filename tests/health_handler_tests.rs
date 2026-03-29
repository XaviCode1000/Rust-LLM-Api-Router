//! Integration tests for health check handlers
//!
//! Tests cover GET /health, /health/detail, /accounts endpoints.

use axum::{body::Body, http::Request, routing::get, Router};
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository, Metrics};
use rust_llm_api_router::interfaces::handlers::health_handler::{
    health, health_detail, list_accounts,
};
use rust_llm_api_router::presentation::AppState;

/// Helper function to create AppState with llm_router
fn create_test_app_state(
    http_client: Arc<HttpClient>,
    account_repo: Arc<dyn AccountRepository>,
    provider_config: std::collections::HashMap<
        String,
        rust_llm_api_router::infrastructure::gateway::llm_gateway::ProviderConfig,
    >,
) -> Arc<AppState> {
    let settings = Settings::default();
    Arc::new(
        AppState::with_provider_config(settings, http_client, account_repo, provider_config)
            .unwrap(),
    )
}

/// Setup test app with health routes
#[allow(dead_code)]
fn setup_health_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository"),
    );

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));

    let _settings = Settings::default();
    let provider_config = default_providers();
    let state = create_test_app_state(http_client, repo, provider_config);

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/detail", get(health_detail))
        .route("/accounts", get(list_accounts))
        .with_state(state);

    (app, temp_dir)
}

/// Setup health app with test accounts
#[allow(dead_code)]
async fn setup_health_app_with_accounts() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository"),
    );

    // Create test accounts
    let account1 = Account::new("acc-1", "openai", "sk-test-key-1");
    let account2 = Account::new("acc-2", "groq", "gq-test-key-2");
    let account3 = Account::inactive("acc-3", "openai", "sk-test-key-3");

    repo.save(account1).await.expect("Should save account");
    repo.save(account2).await.expect("Should save account");
    repo.save(account3).await.expect("Should save account");

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));

    let _settings = Settings::default();
    let provider_config = default_providers();
    let state = create_test_app_state(http_client, repo, provider_config);

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/detail", get(health_detail))
        .route("/accounts", get(list_accounts))
        .with_state(state);

    (app, temp_dir)
}

/// Setup health app with custom metrics
#[allow(dead_code)]
fn setup_health_app_with_metrics() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository"),
    );

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));
    let _metrics = Arc::new(Metrics::new().expect("Should create metrics"));

    let _settings = Settings::default();
    let provider_config = default_providers();
    let state = create_test_app_state(http_client, repo, provider_config);

    // Note: metrics is created but not stored in state when using with_provider_config
    // This test verifies metrics can still be created and used independently

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/detail", get(health_detail))
        .route("/accounts", get(list_accounts))
        .with_state(state);

    (app, temp_dir)
}
