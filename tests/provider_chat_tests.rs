//! Tests para funciones chat() de providers
//!
//! Tests para verificar la implementación de las funciones chat()
//! en OpenAI, Groq y Anthropic providers usando wiremock.

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header, header_exists};
use serde_json::json;

use rust_llm_api_router::domain::entities::{OpenAIChatRequest, OpenAIMessage};
use rust_llm_api_router::domain::traits::LlmProvider;
use rust_llm_api_router::infrastructure::http_client::HttpClient;
use rust_llm_api_router::infrastructure::provider::{OpenAiProvider, GroqProvider, AnthropicProvider};
use std::sync::Arc;

// ============================================================================
// OpenAI Provider Chat Tests
// ============================================================================

#[tokio::test]
async fn test_openai_chat_function_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-test-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-test-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new(
        "gpt-4",
        vec![OpenAIMessage::user("Hello")],
    );
    
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "chatcmpl-test-123");
    assert_eq!(response.model, "gpt-4");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
    assert_eq!(response.usage.prompt_tokens, 10);
    assert_eq!(response.usage.completion_tokens, 15);
    assert_eq!(response.usage.total_tokens, 25);
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_chat_function_with_system_message() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-system-test",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "I am a helpful assistant."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 8,
                "total_tokens": 28
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-test-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new(
        "gpt-4",
        vec![
            OpenAIMessage::system("You are a helpful assistant."),
            OpenAIMessage::user("Who are you?"),
        ],
    );
    
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "chatcmpl-system-test");
    assert_eq!(response.choices[0].message.content, "I am a helpful assistant.");
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_chat_function_error_401() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "message": "Invalid API key",
                "type": "authentication_error"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("gpt-4", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401"));
    assert!(error_msg.contains("Invalid API key"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_chat_function_error_500() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-test-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("gpt-4", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_chat_function_error_429_rate_limit() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "message": "Rate limit exceeded",
                "type": "rate_limit"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-test-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("gpt-4", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("429"));
    assert!(error_msg.contains("Rate limit exceeded"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_openai_chat_function_multiple_choices() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-multi-choice",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "First choice"
                    },
                    "finish_reason": "stop"
                },
                {
                    "index": 1,
                    "message": {
                        "role": "assistant",
                        "content": "Second choice"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        mock_server.uri(),
        "sk-test-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("gpt-4", vec![OpenAIMessage::user("Hello")]);
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "chatcmpl-multi-choice");
    assert_eq!(response.choices.len(), 2);
    assert_eq!(response.choices[0].message.content, "First choice");
    assert_eq!(response.choices[1].message.content, "Second choice");
    
    mock_server.verify().await;
}

// ============================================================================
// Groq Provider Chat Tests
// ============================================================================

#[tokio::test]
async fn test_groq_chat_function_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "groq-chat-test",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama-3.1-70b-versatile",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Groq is fast!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 8,
                "total_tokens": 13
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new(
        "llama-3.1-70b-versatile",
        vec![OpenAIMessage::user("Hello")],
    );
    
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "groq-chat-test");
    assert_eq!(response.model, "llama-3.1-70b-versatile");
    assert_eq!(response.choices[0].message.content, "Groq is fast!");
    assert_eq!(response.usage.total_tokens, 13);
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_chat_function_error_401() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "message": "Invalid API key",
                "type": "authentication_error"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("llama-3.1-70b", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_groq_chat_function_error_500() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        mock_server.uri(),
        "sk-groq-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("llama-3.1-70b", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));
    
    mock_server.verify().await;
}

// ============================================================================
// Anthropic Provider Chat Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_chat_function_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "sk-anthropic-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg-anthropic-test-123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Hello from Claude!"
                }
            ],
            "model": "claude-3-sonnet-20240229",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 12,
                "output_tokens": 10
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new(
        "claude-3-sonnet-20240229",
        vec![OpenAIMessage::user("Hello")],
    );
    
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "msg-anthropic-test-123");
    assert_eq!(response.model, "claude-3-sonnet-20240229");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "Hello from Claude!");
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].finish_reason, Some("end_turn".to_string()));
    assert_eq!(response.usage.prompt_tokens, 12);
    assert_eq!(response.usage.completion_tokens, 10);
    assert_eq!(response.usage.total_tokens, 22);
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_chat_function_with_system_message() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg-system-test",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "I am Claude, a helpful assistant."
                }
            ],
            "model": "claude-3-sonnet-20240229",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 15,
                "output_tokens": 12
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new(
        "claude-3-sonnet-20240229",
        vec![
            OpenAIMessage::system("You are Claude, a helpful assistant."),
            OpenAIMessage::user("Who are you?"),
        ],
    );
    
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "msg-system-test");
    assert_eq!(response.choices[0].message.content, "I am Claude, a helpful assistant.");
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_chat_function_error_401() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "type": "authentication_error",
                "message": "Invalid API key"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-invalid-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("claude-3", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401"));
    assert!(error_msg.contains("Invalid API key"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_chat_function_error_500() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("claude-3", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_chat_function_error_429_rate_limit() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "type": "rate_limit",
                "message": "Rate limit exceeded"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("claude-3", vec![OpenAIMessage::user("Hello")]);
    let result = provider.chat(&request).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("429"));
    assert!(error_msg.contains("Rate limit exceeded"));
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_anthropic_chat_function_empty_content() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg-empty-content",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-3-sonnet-20240229",
            "stop_reason": "stop_sequence",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 0
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        mock_server.uri(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    let request = OpenAIChatRequest::new("claude-3", vec![OpenAIMessage::user("Hello")]);
    let response = provider.chat(&request).await.unwrap();
    
    assert_eq!(response.id, "msg-empty-content");
    // Empty content should default to empty string, not panic
    assert_eq!(response.choices[0].message.content, "");
    assert_eq!(response.usage.completion_tokens, 0);
    
    mock_server.verify().await;
}

// ============================================================================
// Provider Creation Tests
// ============================================================================

#[test]
fn test_openai_provider_creation() {
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = OpenAiProvider::new(
        "https://api.openai.com".to_string(),
        "sk-test-key".to_string(),
        client,
    );
    
    assert_eq!(provider.name(), "openai");
}

#[test]
fn test_groq_provider_creation() {
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = GroqProvider::new(
        "https://api.groq.com".to_string(),
        "sk-groq-key".to_string(),
        client,
    );
    
    assert_eq!(provider.name(), "groq");
}

#[test]
fn test_anthropic_provider_creation() {
    let client = Arc::new(HttpClient::new().unwrap());
    let provider = AnthropicProvider::new(
        "https://api.anthropic.com".to_string(),
        "sk-anthropic-key".to_string(),
        client,
    );
    
    assert_eq!(provider.name(), "anthropic");
}
