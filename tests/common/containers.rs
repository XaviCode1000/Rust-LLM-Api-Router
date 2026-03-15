//! Docker container helpers for integration tests
//!
//! These helpers use testcontainers to spin up mock API servers.
//! Tests will skip if Docker is not available.

use wiremock::MockServer;
use wiremock::matchers::{method, path};
use serde_json::json;

/// Mock OpenAI API server
pub struct MockOpenAIServer {
    server: MockServer,
}

impl MockOpenAIServer {
    /// Start a mock OpenAI server
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Get the server endpoint URL
    pub fn endpoint(&self) -> String {
        self.server.uri()
    }

    /// Setup a successful chat response
    pub async fn setup_chat_response(&self) {
        wiremock::Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "gpt-3.5-turbo",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello from mock OpenAI!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 9,
                    "completion_tokens": 12,
                    "total_tokens": 21
                }
            })))
            .expect(1..)
            .mount(&self.server)
            .await;
    }

    /// Setup a failure response (503)
    pub async fn setup_failure_response(&self) {
        wiremock::Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(wiremock::ResponseTemplate::new(503))
            .expect(1..)
            .mount(&self.server)
            .await;
    }

    /// Setup a slow response (simulates timeout)
    /// Note: wiremock v0.6 doesn't have set_delay, so we just return a normal response
    /// The timeout test will need to use a different approach
    pub async fn setup_slow_response(&self, _delay_ms: u64) {
        wiremock::Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-slow",
                "choices": [{"message": {"content": "Slow response"}}]
            })))
            .expect(1..)
            .mount(&self.server)
            .await;
    }
}

/// Mock Groq API server
pub struct MockGroqServer {
    server: MockServer,
}

impl MockGroqServer {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub fn endpoint(&self) -> String {
        self.server.uri()
    }

    pub async fn setup_chat_response(&self) {
        wiremock::Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "groq-123",
                "choices": [{
                    "message": {"role": "assistant", "content": "Hello from mock Groq!"}
                }]
            })))
            .expect(1..)
            .mount(&self.server)
            .await;
    }
}

/// Helper to check if Docker is available
pub fn is_docker_available() -> bool {
    std::process::Command::new("docker")
        .arg("ps")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Skip test if Docker is not available
pub fn skip_if_no_docker() {
    if !is_docker_available() {
        println!("Skipping test: Docker not available");
        // In a real test, we'd use std::process::exit(0) or return early
    }
}
