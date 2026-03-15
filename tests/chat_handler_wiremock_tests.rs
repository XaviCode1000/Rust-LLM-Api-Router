//! Integration tests for chat handler with wiremock HTTP mocks
//!
//! These tests use wiremock to simulate HTTP responses from LLM providers,
//! allowing us to test the chat handler logic without real API calls.
//!
//! Coverage goal: 17% → 80%+

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::infrastructure::{
    HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics,
};
use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
use rust_llm_api_router::presentation::AppState;

/// Setup: Create app with mock HTTP client pointing to mock server
async fn setup_test_app_with_mock_provider(mock_server: &MockServer) -> (axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    // Setup repository with test account
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    // Create test account - provider will be routed to mock server
    let account = Account::new("mock-account-1", "openai", "sk-mock-key-123");
    repo.save(account).await.unwrap();

    // Create HTTP client with mock URL - this makes all requests go to wiremock
    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());

    let llm_gateway = Arc::new(LlmGatewayImpl::new(http_client.clone(), repo.clone(), 3600));

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

    let app = axum::Router::new()
        .route(
            "/v1/chat/completions",
            axum::routing::post(chat_completions),
        )
        .with_state(state);

    (app, temp_dir)
}

/// Helper to create success mock response
fn create_success_mock_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "id": "chatcmpl-mock-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello from mock provider!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    }))
}

/// Helper to create error mock response
fn create_error_mock_response(status: u16, error_type: &str, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({
        "error": {
            "message": message,
            "type": error_type
        }
    }))
}

// ============================================================================
// SUCCESS SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_chat_handler_success_with_mock_provider() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup mock response (success)
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

    // Make request
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

    // Should succeed
    assert_eq!(response.status(), StatusCode::OK);

    // Verify mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_success_with_temperature_parameter() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup mock response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

    // Make request with temperature
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
                        "temperature": 0.7
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_success_with_max_tokens_parameter() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

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
                        "max_tokens": 512
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_success_with_multiple_messages() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

// ============================================================================
// FAILOVER SCENARIOS (503 errors)
// ============================================================================

#[tokio::test]
async fn test_chat_handler_provider_503_failover() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // First call fails with 503
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Make request - will fail because no other accounts
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

    // Should return BAD_GATEWAY when provider fails
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_provider_502_bad_gateway() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(502))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_provider_504_gateway_timeout() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(504))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

// ============================================================================
// RATE LIMITING (429)
// ============================================================================

#[tokio::test]
async fn test_chat_handler_rate_limit_429() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_error_mock_response(
            429,
            "rate_limit_error",
            "Rate limit exceeded",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    // Should return BAD_GATEWAY when provider returns 429
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

// ============================================================================
// AUTHENTICATION ERRORS (401)
// ============================================================================

#[tokio::test]
async fn test_chat_handler_auth_error_401() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_error_mock_response(
            401,
            "invalid_request_error",
            "Invalid API key",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    // Should return BAD_GATEWAY when auth fails
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_forbidden_403() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_error_mock_response(
            403,
            "forbidden",
            "Access forbidden",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

// ============================================================================
// TIMEOUT SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_chat_handler_timeout() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    use std::time::Duration;

    // Setup very slow response (will timeout)
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .expect(0..) // May or may not be called depending on timeout
        .mount(&mock_server)
        .await;

    // Make request (should timeout or return error)
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
        .await;

    // Should handle timeout gracefully (error or gateway timeout)
    assert!(response.is_err() || response.unwrap().status() == StatusCode::BAD_GATEWAY);
}

// ============================================================================
// STREAMING RESPONSES
// ============================================================================

#[tokio::test]
async fn test_chat_handler_streaming_response() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup streaming response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw(
                    "data: {\"choices\": [{\"delta\": {\"content\": \"Hello\"}}]}\n\n",
                    "text/event-stream",
                ),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    // Make streaming request
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

    // Should return OK with SSE content type
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/event-stream"));

    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_streaming_with_multiple_chunks() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup streaming response with multiple chunks
    let streaming_body = r#"data: {"choices": [{"delta": {"content": "Hello"}}]}

data: {"choices": [{"delta": {"content": " from"}}]}

data: {"choices": [{"delta": {"content": " mock!"}}]}

data: [DONE]

"#;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw(streaming_body, "text/event-stream"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

// ============================================================================
// INVALID REQUEST SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_chat_handler_invalid_model_name_400() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Mock returns 400 for invalid model
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_error_mock_response(
            400,
            "invalid_request_error",
            "Invalid model",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:invalid-model-xyz-123",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return BAD_GATEWAY when provider rejects model
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_empty_message_array_400() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Mock returns 400 for empty messages
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(create_error_mock_response(
            400,
            "invalid_request_error",
            "Messages array cannot be empty",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4",
                        "messages": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_missing_required_field() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Request without messages field - will fail validation before reaching mock
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "openai:gpt-4"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return UNPROCESSABLE_ENTITY or BAD_REQUEST (validation error before mock)
    assert!(
        response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

// ============================================================================
// PROVIDER-SPECIFIC ERROR FORMATS
// ============================================================================

#[tokio::test]
async fn test_chat_handler_openai_error_format() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // OpenAI-style error response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": {
                "message": "This model's maximum context length is 8192 tokens",
                "type": "invalid_request_error",
                "param": "messages",
                "code": "context_length_exceeded"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_generic_provider_error_format() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Generic error response format
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": {
                "type": "invalid_request_error",
                "message": "The request is invalid"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

// ============================================================================
// RETRY LOGIC SCENARIOS (document current behavior)
// ============================================================================

#[tokio::test]
async fn test_chat_handler_single_request_on_error() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup mock to fail - current implementation doesn't retry
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1) // Only called once - no retry
        .mount(&mock_server)
        .await;

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

    // Current behavior: returns BAD_GATEWAY on first failure, no retry
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

// ============================================================================
// CIRCUIT BREAKER STATE TRANSITIONS
// ============================================================================

#[tokio::test]
async fn test_chat_handler_consecutive_failures() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Setup mock to always fail
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503))
        .expect(3..) // At least 3 times
        .mount(&mock_server)
        .await;

    // Make multiple requests
    for _ in 0..3 {
        let response = app
            .clone()
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

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    mock_server.verify().await;
}

// ============================================================================
// HEADER VALIDATION
// ============================================================================

#[tokio::test]
async fn test_chat_handler_authorization_header_sent() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Verify Authorization header is sent correctly
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer sk-mock-key-123"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_content_type_header_sent() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Verify Content-Type header is sent
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Content-Type", "application/json"))
        .respond_with(create_success_mock_response())
        .expect(1)
        .mount(&mock_server)
        .await;

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

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

// ============================================================================
// RESPONSE PARSING EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_chat_handler_malformed_response_body() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Return invalid JSON
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json {{{"))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    // Should handle malformed response gracefully
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_empty_choices_array() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Return response with empty choices
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-empty",
            "object": "chat.completion",
            "choices": [],
            "usage": {"prompt_tokens": 10, "completion_tokens": 0, "total_tokens": 10}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    // Should handle empty choices (may panic or return error)
    // This tests edge case handling
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}

#[tokio::test]
async fn test_chat_handler_missing_usage_in_response() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_test_app_with_mock_provider(&mock_server).await;

    // Return response without usage field
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-no-usage",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hello!"},
                "finish_reason": "stop"
            }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

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

    // Should handle missing usage field gracefully
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_GATEWAY);
    mock_server.verify().await;
}
