//! Domain entities
//!
//! Core business entities representing the domain model.
//! These types are used throughout the application layer.

use serde::{Deserialize, Serialize};

pub mod account;
pub mod account_health;
pub mod openai_types;
pub mod provider;

pub use account::Account;
pub use account_health::{AccountHealth, CircuitBreakerState};
pub use openai_types::{
    OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice, OpenAIError, OpenAIErrorResponse,
    OpenAIMessage, OpenAIUsage,
};
pub use provider::Provider;

/// Chat request sent to an LLM provider.
///
/// # Fields
/// * `model` - The model identifier (e.g., "gpt-4", "claude-3")
/// * `messages` - List of conversation messages
/// * `temperature` - Sampling temperature (0.0 to 2.0)
/// * `max_tokens` - Maximum tokens to generate
/// * `stream` - Whether to stream the response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

impl ChatRequest {
    /// Creates a new `ChatRequest` with required fields.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
        }
    }

    /// Sets the temperature parameter.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the max_tokens parameter.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the stream parameter.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

/// Alias for backward compatibility.
pub type LlmRequest = ChatRequest;

/// Chat response from an LLM provider.
///
/// # Fields
/// * `id` - Unique response identifier
/// * `choices` - List of response choices
/// * `usage` - Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

/// Alias for backward compatibility.
pub type LlmResponse = ChatResponse;

/// A single choice in the chat response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

/// Message in a conversation.
///
/// # Fields
/// * `role` - Role of the message sender ("system", "user", "assistant")
/// * `content` - Content of the message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    /// Creates a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Token usage information from an LLM response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl Usage {
    /// Creates a new `Usage` with all fields.
    pub fn new(prompt_tokens: u32, completion_tokens: u32, total_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }
}

/// Model information from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider_id: String,
}

impl Model {
    /// Creates a new `Model`.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider_id: provider_id.into(),
        }
    }
}
