//! Integration tests for health check handlers
//!
//! Tests cover GET /health, /health/detail, /accounts endpoints.

use axum::{body::Body, http::Request, routing::get, Router};
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;

use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::interfaces::handlers::health_handler::{
    health, health_detail, list_accounts,
};
use rust_llm_api_router::presentation::AppState;
use rust_llm_api_router::config::Settings;

/// Setup test app with health routes
fn setup_health_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path())
            .expect("Should create repository"),
    );

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));
    let metrics = Arc::new(Metrics::new().expect("Should create metrics"));

    let llm_gateway = Arc::new(LlmGatewayImpl::new(
        http_client.clone(),
        repo.clone(),
        3600,
    ));

    let settings = Settings::default();
    let provider_config = Arc::new(default_providers());
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/detail", get(health_detail))
        .route("/accounts", get(list_accounts))
        .with_state(state);

    (app, temp_dir)
}

/// Setup health app with test accounts
async fn setup_health_app_with_accounts() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path())
            .expect("Should create repository"),
    );

    // Create test accounts
    let account1 = Account::new("acc-1", "openai", "sk-test-key-1");
    let account2 = Account::new("acc-2", "groq", "gq-test-key-2");
    let account3 = Account::inactive("acc-3", "openai", "sk-test-key-3");

    repo.save(account1).await.expect("Should save account");
    repo.save(account2).await.expect("Should save account");
    repo.save(account3).await.expect("Should save account");

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));
    let metrics = Arc::new(Metrics::new().expect("Should create metrics"));

    let llm_gateway = Arc::new(LlmGatewayImpl::new(
        http_client.clone(),
        repo.clone(),
        3600,
    ));

    let settings = Settings::default();
    let provider_config = Arc::new(default_providers());
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = Router::new()
        .route("/health/detail", get(health_detail))
        .with_state(state);

    (app, temp_dir)
}

/// Setup accounts app with test data
async fn setup_accounts_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path())
            .expect("Should create repository"),
    );

    let account = Account::new("test-acc-1", "openai", "sk-test-key-12345678");
    repo.save(account).await.expect("Should save account");

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));
    let metrics = Arc::new(Metrics::new().expect("Should create metrics"));

    let llm_gateway = Arc::new(LlmGatewayImpl::new(
        http_client.clone(),
        repo.clone(),
        3600,
    ));
    
    let settings = Settings::default();
    let provider_config = Arc::new(default_providers());
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = Router::new()
        .route("/accounts", get(list_accounts))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_health_check_returns_ok() {
    let (app, _temp) = setup_health_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["timestamp"].as_u64().is_some());
    assert_eq!(json["version"], "0.1.0");
}

#[tokio::test]
async fn test_health_detail_returns_ok() {
    let (app, _temp) = setup_health_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health/detail")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify response structure
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["providers"]["total"].as_u64().is_some());
    assert!(json["accounts"]["total"].as_u64().is_some());
}

#[tokio::test]
async fn test_health_detail_with_accounts() {
    let (app, _temp) = setup_health_app_with_accounts().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health/detail")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify account counts
    assert_eq!(json["accounts"]["total"].as_u64().unwrap(), 3);
    assert_eq!(json["accounts"]["active"].as_u64().unwrap(), 2);
    assert_eq!(json["accounts"]["inactive"].as_u64().unwrap(), 1);

    // Verify provider counts
    assert_eq!(json["providers"]["total"].as_u64().unwrap(), 2);
    assert_eq!(json["providers"]["enabled"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn test_list_accounts_empty() {
    let (app, _temp) = setup_health_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_accounts_with_data() {
    let (app, _temp) = setup_accounts_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.is_array());
    let accounts = json.as_array().unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["id"], "test-acc-1");
    assert_eq!(accounts[0]["provider_id"], "openai");
    assert_eq!(accounts[0]["is_active"], true);
    // API key should be truncated to prefix
    assert_eq!(accounts[0]["api_key_prefix"], "sk-test-");
}

#[tokio::test]
async fn test_health_check_content_type() {
    let (app, _temp) = setup_health_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );
}

#[tokio::test]
async fn test_health_detail_content_type() {
    let (app, _temp) = setup_health_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health/detail")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );
}
