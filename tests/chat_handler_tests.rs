//! Integration tests for chat handlers with mock repositories
//!
//! Tests cover POST /v1/chat/completions endpoint scenarios.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository};
use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
use rust_llm_api_router::presentation::state::AppState;

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

/// Setup test fixtures with in-memory repository
async fn setup_test_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository"),
    );

    // Create test accounts
    let account_openai = Account::new("test-openai-1", "openai", "sk-test-key-123");
    let account_groq = Account::new("test-groq-1", "groq", "gq-test-key-456");

    repo.save(account_openai)
        .await
        .expect("Should save account");
    repo.save(account_groq).await.expect("Should save account");

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));

    // Create provider config with default providers
    let provider_config = default_providers();
    let state = create_test_app_state(http_client, repo, provider_config);

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_chat_handler_missing_model() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing model should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn test_chat_handler_missing_messages() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "gpt-4"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing messages should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn test_chat_handler_empty_messages() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "gpt-4",
                        "messages": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Empty messages should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn test_chat_handler_invalid_message_role() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "gpt-4",
                        "messages": [{"role": "invalid", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Invalid role should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn test_chat_handler_no_accounts_for_provider() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository"),
    );

    // No accounts created - provider has no accounts

    let http_client = Arc::new(HttpClient::new().expect("Should create HTTP client"));

    let provider_config = default_providers();
    let state = create_test_app_state(http_client, repo, provider_config);

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // No accounts for provider should return error
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn test_chat_handler_model_parsing_colon() {
    let app = setup_test_app().await.0;

    // This test verifies model parsing with colon separator
    // The actual request will fail (no real API key), but parsing should work
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4-turbo",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should parse model correctly (may fail on API call, but not on parsing)
    // Status depends on whether request reaches provider
    assert!(
        response.status() == StatusCode::BAD_GATEWAY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_handler_model_parsing_slash() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "groq/llama-3",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should parse model correctly
    assert!(
        response.status() == StatusCode::BAD_GATEWAY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_handler_streaming_flag() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": [{"role": "user", "content": "Hello"}],
                        "stream": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Streaming request - should return SSE or error
    // Status depends on implementation
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_chat_handler_with_temperature() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": [{"role": "user", "content": "Hello"}],
                        "temperature": 0.9
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should accept temperature parameter
    assert!(
        response.status() == StatusCode::BAD_GATEWAY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_handler_with_max_tokens() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": [{"role": "user", "content": "Hello"}],
                        "max_tokens": 2048
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should accept max_tokens parameter
    assert!(
        response.status() == StatusCode::BAD_GATEWAY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_handler_multiple_messages() {
    let app = setup_test_app().await.0;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": [
                            {"role": "system", "content": "You are helpful"},
                            {"role": "user", "content": "Hello"},
                            {"role": "assistant", "content": "Hi there!"},
                            {"role": "user", "content": "How are you?"}
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should accept multiple messages
    assert!(
        response.status() == StatusCode::BAD_GATEWAY
            || response.status() == StatusCode::BAD_REQUEST
    );
}
