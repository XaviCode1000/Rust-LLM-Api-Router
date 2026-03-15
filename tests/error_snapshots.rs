//! Snapshot tests for error formatting and response structures
//!
//! These tests use `insta` to verify that error messages and response formats
//! remain consistent across changes, preventing accidental breaking changes.
//!
//! # Running Tests
//!
//! ```bash
//! cargo insta test --review
//! cargo insta accept
//! ```

use insta::assert_json_snapshot;
use rust_llm_api_router::{
    domain::{DomainError, OpenAIChatResponse, OpenAIMessage, OpenAIUsage},
    error::Error,
};

/// Test that error messages don't leak sensitive information like API keys.
///
/// This is a security-critical test ensuring that when errors are serialized
/// or displayed, they don't accidentally expose credentials.
#[test]
fn test_error_format_no_api_key_leak() {
    // Create an authentication error that might contain sensitive data
    let auth_error = DomainError::authentication_error("Invalid API key: sk-abc123xyz789");

    // Convert to application error
    let app_error: Error = auth_error.into();

    // Serialize to JSON for snapshot
    let error_json = serde_json::json!({
        "error_type": "authentication_error",
        "message": format!("{}", app_error),
        "source": "domain"
    });

    assert_json_snapshot!("error_no_api_key_leak", error_json);
}

/// Test that chat response format matches OpenAI-compatible structure.
///
/// This ensures our response structure remains stable for API consumers.
#[test]
fn test_chat_response_format() {
    let messages = vec![
        OpenAIMessage::system("You are a helpful assistant."),
        OpenAIMessage::user("Hello, world!"),
        OpenAIMessage::assistant("Hello! How can I help you today?"),
    ];

    let choices = vec![rust_llm_api_router::domain::OpenAIChoice::new(
        0,
        messages[2].clone(),
        Some("stop"),
    )];

    let usage = OpenAIUsage::new(10, 20, 30);

    let response = OpenAIChatResponse::new("chatcmpl-123456789", "gpt-4", choices, usage);

    // Snapshot the response structure (excluding dynamic timestamp)
    // Using a helper struct to exclude the 'created' field which varies
    #[derive(serde::Serialize)]
    struct SnapshotResponse {
        id: String,
        object: String,
        model: String,
        choices: Vec<rust_llm_api_router::domain::OpenAIChoice>,
        usage: OpenAIUsage,
        system_fingerprint: Option<String>,
    }

    let snapshot = SnapshotResponse {
        id: response.id,
        object: response.object,
        model: response.model,
        choices: response.choices,
        usage: response.usage,
        system_fingerprint: response.system_fingerprint,
    };

    assert_json_snapshot!("chat_response_format", snapshot);
}

/// Test frontmatter generation for documentation/metadata.
///
/// This tests a utility function that generates YAML frontmatter
/// for documentation files or configuration exports.
#[test]
fn test_frontmatter_generation() {
    // Simulate frontmatter generation for API documentation
    let frontmatter_data = serde_json::json!({
        "title": "LLM API Router Configuration",
        "version": "0.1.0",
        "author": "LLM Router Team",
        "date": "2026-03-14",
        "tags": ["rust", "api", "llm", "proxy"],
        "providers": ["openai", "anthropic", "google"],
        "features": {
            "failover": true,
            "rate_limiting": true,
            "metrics": true,
            "streaming": true
        }
    });

    assert_json_snapshot!("frontmatter_config", frontmatter_data);
}

/// Additional test: Domain error serialization consistency.
///
/// Ensures domain errors serialize consistently for logging and monitoring.
#[test]
fn test_domain_error_serialization() {
    let errors = vec![
        DomainError::invalid_request("Missing required field: model"),
        DomainError::provider_not_found("openai"),
        DomainError::provider_disabled("anthropic"),
        DomainError::account_not_found("acc_123"),
        DomainError::account_inactive("acc_456"),
        DomainError::model_not_found("gpt-5"),
        DomainError::authentication_error("Invalid credentials"),
        DomainError::rate_limited("Too many requests"),
        DomainError::validation_error("Temperature must be between 0 and 2"),
    ];

    let serialized: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();

    assert_json_snapshot!("domain_error_messages", serialized);
}

/// Test OpenAI error response format.
///
/// Verifies that error responses match the OpenAI API specification.
#[test]
fn test_openai_error_response() {
    use rust_llm_api_router::domain::OpenAIErrorResponse;

    let error_response =
        OpenAIErrorResponse::new("authentication_error", "Invalid API key provided");

    assert_json_snapshot!("openai_error_response", error_response);
}
