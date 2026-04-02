//! Additional unit tests for chat_handler.rs to reach 80%+ coverage
//!
//! These tests cover functions not exercised by wiremock integration tests:
//! - list_models handler
//! - get_api_key_for_models helper
//! - convert_to_openai_response
//! - stream_to_sse_events edge cases
//! - parse_model variations

use axum::http::StatusCode;
use rust_llm_api_router::config::Settings;
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::domain::{ChatResponse, Choice, Message, Usage};
use rust_llm_api_router::infrastructure::gateway::llm_gateway::default_providers;
use rust_llm_api_router::infrastructure::{HttpClient, JsonAccountRepository, Metrics};
use rust_llm_api_router::interfaces::handlers::chat_handler::{
    convert_to_openai_response, get_api_key_for_models, list_models, parse_model, OpenAIModelInfo,
    OpenAIModelsResponse,
};
use rust_llm_api_router::presentation::state::AppState;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// PARSE_MODEL TESTS
// ============================================================================

#[test]
fn test_parse_model_colon_separator() {
    let (provider, model) = parse_model("openai:gpt-4");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4");
}

#[test]
fn test_parse_model_slash_separator() {
    let (provider, model) = parse_model("anthropic/claude-3");
    assert_eq!(provider, "anthropic");
    assert_eq!(model, "claude-3");
}

#[test]
fn test_parse_model_default_no_separator() {
    let (provider, model) = parse_model("gpt-4");
    assert_eq!(provider, "default");
    assert_eq!(model, "gpt-4");
}

#[test]
fn test_parse_model_multiple_separators() {
    let (provider, model) = parse_model("openai:gpt-4:turbo");
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4:turbo");
}

#[test]
fn test_parse_model_empty_string() {
    let (provider, model) = parse_model("");
    assert_eq!(provider, "default");
    assert_eq!(model, "");
}

// ============================================================================
// CONVERT_TO_OPENAI_RESPONSE TESTS
// ============================================================================

#[test]
fn test_convert_to_openai_response_basic() {
    let chat_response = ChatResponse {
        id: "mock-123".to_string(),
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

    let openai_response = convert_to_openai_response(chat_response, "gpt-4");

    assert_eq!(openai_response.id, "chatcmpl-mock-123");
    assert_eq!(openai_response.model, "gpt-4");
    assert_eq!(openai_response.choices.len(), 1);
    assert_eq!(openai_response.choices[0].message.content, "Hello!");
    assert_eq!(openai_response.choices[0].finish_reason, Some("stop".to_string()));
    assert_eq!(openai_response.usage.prompt_tokens, 10);
    assert_eq!(openai_response.usage.completion_tokens, 5);
    assert_eq!(openai_response.usage.total_tokens, 15);
}

#[test]
fn test_convert_to_openai_response_multiple_choices() {
    let chat_response = ChatResponse {
        id: "multi-choice".to_string(),
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

    let openai_response = convert_to_openai_response(chat_response, "gpt-4");

    assert_eq!(openai_response.choices.len(), 2);
    assert_eq!(openai_response.choices[0].message.content, "First");
    assert_eq!(openai_response.choices[1].message.content, "Second");
}

#[test]
fn test_convert_to_openai_response_no_finish_reason() {
    let chat_response = ChatResponse {
        id: "no-finish".to_string(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: "Hello".to_string(),
            },
            finish_reason: None,
        }],
        usage: Usage {
            prompt_tokens: 5,
            completion_tokens: 5,
            total_tokens: 10,
        },
    };

    let openai_response = convert_to_openai_response(chat_response, "gpt-4");

    assert_eq!(openai_response.choices[0].finish_reason, None);
}

#[test]
fn test_convert_to_openai_response_empty_choices() {
    let chat_response = ChatResponse {
        id: "empty-choices".to_string(),
        choices: vec![],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    let openai_response = convert_to_openai_response(chat_response, "gpt-4");

    assert!(openai_response.choices.is_empty());
}

// ============================================================================
// LIST_MODELS TESTS
// ============================================================================

async fn setup_app_state_with_accounts_in_temp(
    temp_dir: &TempDir,
    accounts: Vec<Account>,
) -> Arc<AppState> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());

    for account in accounts {
        repo.save(account).await.unwrap();
    }

    let http_client = Arc::new(HttpClient::new().unwrap());
    let _metrics = Arc::new(Metrics::new().unwrap());

    let provider_config = default_providers();
    let settings = Settings::default();

    // Use the with_provider_config constructor which handles llm_router
    Arc::new(AppState::with_provider_config(settings, http_client, repo, provider_config).unwrap())
}

#[tokio::test]
async fn test_list_models_no_accounts_returns_service_unavailable() {
    let temp_dir = TempDir::new().unwrap();
    let state = setup_app_state_with_accounts_in_temp(&temp_dir, vec![]).await;

    let result = list_models(axum::extract::State(state)).await;

    assert!(result.is_err());
    let (status, _error) = result.unwrap_err();
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_get_api_key_for_models_with_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let accounts = vec![Account::new("account-1", "openai", "sk-key-123")];
    let state = setup_app_state_with_accounts_in_temp(&temp_dir, accounts).await;

    let api_key = get_api_key_for_models(&state).await;

    assert!(api_key.is_some());
    assert_eq!(api_key.unwrap(), "sk-key-123");
}

#[tokio::test]
async fn test_get_api_key_for_models_no_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let state = setup_app_state_with_accounts_in_temp(&temp_dir, vec![]).await;

    let api_key = get_api_key_for_models(&state).await;

    assert!(api_key.is_none());
}

#[tokio::test]
async fn test_get_api_key_for_models_multiple_accounts_returns_first() {
    let temp_dir = TempDir::new().unwrap();
    let accounts = vec![
        Account::new("account-1", "openai", "sk-first-key"),
        Account::new("account-2", "groq", "gq-second-key"),
    ];
    let state = setup_app_state_with_accounts_in_temp(&temp_dir, accounts).await;

    let api_key = get_api_key_for_models(&state).await;

    assert!(api_key.is_some());
    assert_eq!(api_key.unwrap(), "sk-first-key");
}

// ============================================================================
// STREAM_TO_SSE_EVENTS EDGE CASES (via integration)
// ============================================================================

#[tokio::test]
async fn test_streaming_with_empty_chunk() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
    use serde_json::json;
    use tower::util::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let account = Account::new("mock-account", "openai", "sk-mock-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let provider_config = default_providers();
    let state = Arc::new(
        AppState::with_provider_config(Settings::default(), http_client, repo, provider_config)
            .unwrap(),
    );

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(chat_completions))
        .with_state(state);

    // Mock with empty chunk
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw("\n\n", "text/event-stream"),
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

#[tokio::test]
async fn test_streaming_with_valid_utf8_handling() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use rust_llm_api_router::interfaces::handlers::chat_handler::chat_completions;
    use serde_json::json;
    use tower::util::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let repo: Arc<dyn AccountRepository> =
        Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let account = Account::new("mock-account", "openai", "sk-mock-key");
    repo.save(account).await.unwrap();

    let http_client = Arc::new(HttpClient::with_mock_url(&mock_server.uri()).unwrap());
    let provider_config = default_providers();
    let state = Arc::new(
        AppState::with_provider_config(Settings::default(), http_client, repo, provider_config)
            .unwrap(),
    );

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(chat_completions))
        .with_state(state);

    // Mock with valid SSE data that will be processed
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw("data: {\"test\": \"valid\"}\n\n", "text/event-stream"),
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
// OPENAI MODELS RESPONSE STRUCTS
// ============================================================================

#[test]
fn test_openai_models_response_serialization() {
    let models_response = OpenAIModelsResponse {
        object: "list".to_string(),
        data: vec![OpenAIModelInfo {
            id: "openai:gpt-4".to_string(),
            object: "model".to_string(),
            created: 1234567890,
            owned_by: "openai".to_string(),
        }],
    };

    let json = serde_json::to_string(&models_response).unwrap();
    assert!(json.contains("\"object\":\"list\""));
    assert!(json.contains("\"id\":\"openai:gpt-4\""));
}

#[test]
fn test_openai_model_info_clone() {
    let model_info = OpenAIModelInfo {
        id: "gpt-4".to_string(),
        object: "model".to_string(),
        created: 1234567890,
        owned_by: "openai".to_string(),
    };

    let cloned = model_info.clone();
    assert_eq!(cloned.id, model_info.id);
    assert_eq!(cloned.owned_by, model_info.owned_by);
}
