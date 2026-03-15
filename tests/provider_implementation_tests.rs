//! Tests para implementaciones específicas de providers (OpenAI, Groq, Anthropic)
//!
//! Tests adicionales para cubrir list_models() con más casos de error
//! y verificar el comportamiento específico de cada provider.

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use serde_json::json;
use std::sync::Arc;

use rust_llm_api_router::domain::traits::LlmProvider;
use rust_llm_api_router::infrastructure::http_client::HttpClient;
use rust_llm_api_router::infrastructure::provider::openai::OpenAiProvider;
use rust_llm_api_router::infrastructure::provider::groq::GroqProvider;
use rust_llm_api_router::infrastructure::provider::anthropic::AnthropicProvider;

// ============================================================================
// OPENAI PROVIDER TESTS - Additional Coverage
// ============================================================================

#[tokio::test]
async fn test_openai_provider_list_models_empty_response() {
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
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );
    
    let models = provider.list_models("sk-openai-key").await.unwrap();
    
    assert!(models.is_empty());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_list_models_500_server_error() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-openai-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_provider_list_models_403_forbidden() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": {"message": "Forbidden", "type": "forbidden_error"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-openai-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-openai-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

// ============================================================================
// GROQ PROVIDER TESTS - Additional Coverage
// ============================================================================

#[tokio::test]
async fn test_groq_provider_list_models_single_model() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "llama-3.1-70b-versatile", "object": "model", "owned_by": "groq"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );
    
    let models = provider.list_models("sk-groq-key").await.unwrap();
    
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "llama-3.1-70b-versatile");
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_list_models_502_bad_gateway() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(502))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-groq-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_list_models_403_forbidden() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": {"message": "Forbidden"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-groq-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_provider_list_models_many_models() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "llama-3.1-70b-versatile", "object": "model", "owned_by": "groq"},
                {"id": "llama-3.1-8b-instant", "object": "model", "owned_by": "groq"},
                {"id": "mixtral-8x7b-32768", "object": "model", "owned_by": "groq"},
                {"id": "gemma-7b-it", "object": "model", "owned_by": "groq"},
                {"id": "llama3-70b-8192", "object": "model", "owned_by": "groq"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        http_client,
    );
    
    let models = provider.list_models("sk-groq-key").await.unwrap();
    
    assert_eq!(models.len(), 5);
    assert_eq!(models[0].id, "llama-3.1-70b-versatile");
    assert_eq!(models[4].id, "llama3-70b-8192");
    mock_server.verify().await;
}

// ============================================================================
// ANTHROPIC PROVIDER TESTS - Additional Coverage
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_list_models_missing_display_name() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "claude-unknown", "type": "model"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );
    
    let models = provider.list_models("sk-anthropic-key").await.unwrap();
    
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "claude-unknown"); // Falls back to id
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_list_models_503_service_unavailable() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-anthropic-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_list_models_403_forbidden() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": {"type": "forbidden_error", "message": "Forbidden"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );
    
    let result = provider.list_models("sk-anthropic-key").await;
    
    assert!(result.is_err());
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_provider_list_models_with_display_name() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "claude-3-opus-20240229", "type": "model", "display_name": "Claude 3 Opus"},
                {"id": "claude-3-sonnet-20240229", "type": "model", "display_name": "Claude 3 Sonnet"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        http_client,
    );
    
    let models = provider.list_models("sk-anthropic-key").await.unwrap();
    
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].name, "Claude 3 Opus");
    assert_eq!(models[1].name, "Claude 3 Sonnet");
    mock_server.verify().await;
}

// ============================================================================
// PROVIDER NAME TESTS
// ============================================================================

#[test]
fn test_openai_provider_name_constant() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        "https://api.openai.com/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );
    
    assert_eq!(provider.name(), "openai");
}

#[test]
fn test_groq_provider_name_constant() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        "https://api.groq.com/openai/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );
    
    assert_eq!(provider.name(), "groq");
}

#[test]
fn test_anthropic_provider_name_constant() {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        "https://api.anthropic.com/v1".to_string(),
        "sk-key".to_string(),
        http_client,
    );
    
    assert_eq!(provider.name(), "anthropic");
}
