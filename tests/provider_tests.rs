//! Integration tests for LLM providers (OpenAI, Groq, Anthropic)
//!
//! Tests verify correct HTTP headers, endpoints, and response parsing
//! using wiremock for HTTP mocking.

use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use rust_llm_api_router::domain::traits::LlmProvider;
use rust_llm_api_router::infrastructure::http_client::HttpClient;

// ============================================================================
// OpenAI Provider Tests
// ============================================================================

#[tokio::test]
async fn test_openai_provider_list_models_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("Authorization", "Bearer sk-openai-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "gpt-4", "object": "model", "owned_by": "openai"},
                {"id": "gpt-3.5-turbo", "object": "model", "owned_by": "openai"},
                {"id": "gpt-4-turbo", "object": "model", "owned_by": "openai"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );

    let models = provider.list_models("sk-openai-key").await.unwrap();

    assert_eq!(models.len(), 3);
    assert_eq!(models[0].id, "gpt-4");
    assert_eq!(models[0].provider_id, "openai");
    assert_eq!(models[1].id, "gpt-3.5-turbo");
    assert_eq!(models[2].id, "gpt-4-turbo");

    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_list_models_401_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {"message": "Invalid API key", "type": "invalid_request_error"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-invalid-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_list_models_503_service_unavailable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-openai-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_name() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        "https://api.openai.com/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );

    assert_eq!(provider.name(), "openai");
}

// ============================================================================
// Groq Provider Tests
// ============================================================================

#[tokio::test]
async fn test_groq_provider_list_models_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("Authorization", "Bearer sk-groq-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "llama-3.1-70b-versatile", "object": "model", "owned_by": "groq"},
                {"id": "llama-3.1-8b-instant", "object": "model", "owned_by": "groq"},
                {"id": "mixtral-8x7b-32768", "object": "model", "owned_by": "groq"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::groq::GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );

    let models = provider.list_models("sk-groq-key").await.unwrap();

    assert_eq!(models.len(), 3);
    assert_eq!(models[0].id, "llama-3.1-70b-versatile");
    assert_eq!(models[0].provider_id, "groq");
    assert_eq!(models[1].id, "llama-3.1-8b-instant");
    assert_eq!(models[2].id, "mixtral-8x7b-32768");

    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_list_models_401_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {"message": "Invalid API key"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::groq::GroqProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-invalid-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_name() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::groq::GroqProvider::new(
        "https://api.groq.com/openai/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );

    assert_eq!(provider.name(), "groq");
}

// ============================================================================
// Anthropic Provider Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_list_models_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("x-api-key", "sk-anthropic-key"))
        .and(header("anthropic-version", "2024-06-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "claude-3-opus-20240229", "type": "model", "display_name": "Claude 3 Opus"},
                {"id": "claude-3-sonnet-20240229", "type": "model", "display_name": "Claude 3 Sonnet"},
                {"id": "claude-3-haiku-20240307", "type": "model", "display_name": "Claude 3 Haiku"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );

    let models = provider.list_models("sk-anthropic-key").await.unwrap();

    assert_eq!(models.len(), 3);
    assert_eq!(models[0].id, "claude-3-opus-20240229");
    assert_eq!(models[0].provider_id, "anthropic");
    assert_eq!(models[0].name, "Claude 3 Opus");
    assert_eq!(models[1].id, "claude-3-sonnet-20240229");
    assert_eq!(models[2].id, "claude-3-haiku-20240307");

    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_list_models_401_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {"type": "authentication_error", "message": "Invalid API key"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-invalid-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_list_models_429_rate_limit() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {"type": "rate_limit_error", "message": "Rate limit exceeded"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-anthropic-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_name() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider::new(
        "https://api.anthropic.com/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );

    assert_eq!(provider.name(), "anthropic");
}

// ============================================================================
// Provider Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_openai_provider_invalid_json_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-openai-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_missing_data_field() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": [{"id": "gpt-4"}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );

    let result = provider.list_models("sk-openai-key").await;

    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_empty_models_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::groq::GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );

    let models = provider.list_models("sk-groq-key").await.unwrap();

    assert!(models.is_empty());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_models_fallback_to_id_for_name() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("x-api-key", "sk-anthropic-key"))
        .and(header("anthropic-version", "2024-06-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "claude-3-unknown", "type": "model"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );

    let models = provider.list_models("sk-anthropic-key").await.unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "claude-3-unknown");
    assert_eq!(models[0].name, "claude-3-unknown");

    mock_server.verify().await;
}
