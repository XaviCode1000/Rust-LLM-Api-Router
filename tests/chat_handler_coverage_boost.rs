//! Additional tests to push chat_handler.rs coverage to 80%+
//!
//! Focus on remaining branches:
//! - get_provider_base_url for different providers
//! - list_models success path
//! - stream_to_sse_events error cases

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use serde_json::json;
use axum::{http::{Request, StatusCode}, body::Body};
use tower::util::ServiceExt;
use std::sync::Arc;
use tempfile::TempDir;

use rust_llm_api_router::domain::{Account, AccountRepository, ChatResponse, Choice, Message, Usage};
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::interfaces::handlers::chat_handler::{
    chat_completions, parse_model, convert_to_openai_response,
    list_models, get_api_key_for_models, OpenAIModelsResponse, OpenAIModelInfo,
};
use rust_llm_api_router::presentation::state::AppState;
use rust_llm_api_router::config::Settings;

// ============================================================================
// GET_PROVIDER_BASE_URL TESTS (via different provider accounts)
// ============================================================================

async fn setup_app_with_provider(provider: &str, mock_server: &MockServer) -> (axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap(),
    );

    let account = Account::new("mock-account", provider, "sk-mock-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(chat_completions))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_groq_provider_base_url() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_app_with_provider("groq", &mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer sk-mock-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "groq-test",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Groq response"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
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
                .body(Body::from(json!({
                    "model": "groq:llama-3",
                    "messages": [{"role": "user", "content": "Hello"}]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_openrouter_provider_base_url() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_app_with_provider("openrouter", &mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "openrouter-test",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "OpenRouter response"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
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
                .body(Body::from(json!({
                    "model": "openrouter:anthropic/claude-3",
                    "messages": [{"role": "user", "content": "Hello"}]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_mistral_provider_base_url() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_app_with_provider("mistral", &mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "mistral-test",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Mistral response"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
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
                .body(Body::from(json!({
                    "model": "mistral:mistral-large",
                    "messages": [{"role": "user", "content": "Hello"}]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cerebras_provider_base_url() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_app_with_provider("cerebras", &mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "cerebras-test",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Cerebras response"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
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
                .body(Body::from(json!({
                    "model": "cerebras:llama-3.1",
                    "messages": [{"role": "user", "content": "Hello"}]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_anthropic_provider_base_url() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_app_with_provider("anthropic", &mock_server).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "anthropic-test",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Anthropic response"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
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
                .body(Body::from(json!({
                    "model": "anthropic:claude-3-opus",
                    "messages": [{"role": "user", "content": "Hello"}]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// LIST_MODELS TESTS
// ============================================================================

async fn setup_list_models_app(mock_server: &MockServer) -> (axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap(),
    );

    let account = Account::new("test-account", "openai", "sk-test-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = axum::Router::new()
        .route("/v1/models", axum::routing::get(list_models))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_list_models_success() {
    let mock_server = MockServer::start().await;
    let (app, _temp) = setup_list_models_app(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("Authorization", "Bearer sk-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "gpt-4", "name": "GPT-4", "owned_by": "openai"},
                {"id": "gpt-3.5-turbo", "name": "GPT-3.5 Turbo", "owned_by": "openai"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["object"], "list");
    assert!(response_json["data"].is_array());
    assert_eq!(response_json["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_models_no_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap(),
    );

    // No accounts

    let http_client = Arc::new(HttpClient::new().unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = axum::Router::new()
        .route("/v1/models", axum::routing::get(list_models))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return service unavailable when no API keys
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ============================================================================
// STREAM_TO_SSE_EVENTS ERROR CASES
// ============================================================================

#[tokio::test]
async fn test_stream_to_sse_events_invalid_utf8() {
    use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
    use rust_llm_api_router::presentation::state::AppState;
    use rust_llm_api_router::config::Settings;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap(),
    );
    let account = Account::new("mock-account", "openai", "sk-mock-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(chat_completions))
        .with_state(state);

    // Mock that returns binary/invalid UTF-8 data
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Content-Type", "text/event-stream")
            .set_body_raw(vec![0xFF, 0xFE, 0xFD, 0xFC], "text/event-stream"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "model": "openai:gpt-4",
                    "messages": [{"role": "user", "content": "Stream test"}],
                    "stream": true
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify SSE headers
    let headers = response.headers();
    assert_eq!(
        headers.get("content-type").unwrap(),
        "text/event-stream"
    );

    mock_server.verify().await;
}

#[tokio::test]
async fn test_stream_to_sse_events_empty_chunks() {
    use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
    use rust_llm_api_router::presentation::state::AppState;
    use rust_llm_api_router::config::Settings;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap(),
    );
    let account = Account::new("mock-account", "openai", "sk-mock-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let metrics = Arc::new(Metrics::new().unwrap());
    let provider_config = Arc::new(default_providers());
    let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
        http_client.clone(),
        repo.clone(),
        (*provider_config).clone(),
        3600,
    ));
    let settings = Settings::default();
    let state = Arc::new(AppState {
        config: settings,
        http_client,
        metrics,
        account_repo: repo.clone(),
        llm_gateway,
        provider_config,
    });

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(chat_completions))
        .with_state(state);

    // Mock that returns empty chunks
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Content-Type", "text/event-stream")
            .set_body_raw("\n\n\n\n", "text/event-stream"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "model": "openai:gpt-4",
                    "messages": [{"role": "user", "content": "Stream test"}],
                    "stream": true
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    mock_server.verify().await;
}

// ============================================================================
// PARSE_MODEL TESTS
// ============================================================================

#[test]
fn test_parse_model_colon_separator() {
    let (provider, model) = parse_model("openai:gpt-4-turbo");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4-turbo");
}

#[test]
fn test_parse_model_slash_separator() {
    let (provider, model) = parse_model("groq/llama-3");
    assert_eq!(provider, "groq");
    assert_eq!(model, "llama-3");
}

#[test]
fn test_parse_model_no_separator() {
    let (provider, model) = parse_model("gpt-4");
    assert_eq!(provider, "default");
    assert_eq!(model, "gpt-4");
}

#[test]
fn test_parse_model_multiple_separators() {
    let (provider, model) = parse_model("openai:gpt-4-turbo-preview");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4-turbo-preview");
}

// ============================================================================
// CONVERT_TO_OPENAI_RESPONSE TESTS
// ============================================================================

#[test]
fn test_convert_to_openai_response_single_choice() {
    let chat_response = ChatResponse {
        id: "test-123".to_string(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: "Hello!".to_string(),
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };

    let openai_response = convert_to_openai_response(chat_response, "openai:gpt-4");

    assert_eq!(openai_response.id, "chatcmpl-test-123");
    assert_eq!(openai_response.model, "openai:gpt-4");
    assert_eq!(openai_response.choices.len(), 1);
    assert_eq!(openai_response.choices[0].message.content, "Hello!");
    assert_eq!(openai_response.usage.total_tokens, 15);
}

#[test]
fn test_convert_to_openai_response_multiple_choices() {
    let chat_response = ChatResponse {
        id: "test-456".to_string(),
        choices: vec![
            Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: "First".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            },
            Choice {
                index: 1,
                message: Message {
                    role: "assistant".to_string(),
                    content: "Second".to_string(),
                },
                finish_reason: Some("length".to_string()),
            },
        ],
        usage: Usage {
            prompt_tokens: 20,
            completion_tokens: 10,
            total_tokens: 30,
        },
    };

    let openai_response = convert_to_openai_response(chat_response, "groq:llama-3");

    assert_eq!(openai_response.choices.len(), 2);
    assert_eq!(openai_response.choices[0].message.content, "First");
    assert_eq!(openai_response.choices[1].message.content, "Second");
    assert_eq!(openai_response.choices[1].finish_reason, Some("length".to_string()));
}

#[test]
fn test_convert_to_openai_response_no_finish_reason() {
    let chat_response = ChatResponse {
        id: "test-789".to_string(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: "No finish reason".to_string(),
            },
            finish_reason: None,
        }],
        usage: Usage {
            prompt_tokens: 5,
            completion_tokens: 3,
            total_tokens: 8,
        },
    };

    let openai_response = convert_to_openai_response(chat_response, "mistral:mistral-large");

    assert_eq!(openai_response.choices[0].finish_reason, None);
}
