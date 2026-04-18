//! OpenAI-compatible API types
//!
//! These types match the OpenAI Chat Completions API format.
//! See: <https://platform.openai.com/docs/api-reference/chat>

use serde::{Deserialize, Serialize};

/// Chat completion request matching OpenAI API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatRequest {
    /// Model ID to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub model: String,

    /// List of messages in the conversation
    pub messages: Vec<OpenAIMessage>,

    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Top-p sampling (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// User identifier for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl OpenAIChatRequest {
    /// Creates a new request with minimal required fields.
    pub fn new(model: impl Into<String>, messages: Vec<OpenAIMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
            stop: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            user: None,
        }
    }
}

/// A message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    /// Role of the message sender
    pub role: String,

    /// Content of the message
    pub content: String,

    /// Optional name for the sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OpenAIMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            name: None,
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            name: None,
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            name: None,
        }
    }
}

/// Chat completion response matching OpenAI API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatResponse {
    /// Unique response ID
    pub id: String,

    /// Object type (always "chat.completion")
    pub object: String,

    /// Unix timestamp of creation
    pub created: u64,

    /// Model used for generation
    pub model: String,

    /// List of completion choices
    pub choices: Vec<OpenAIChoice>,

    /// Token usage information
    pub usage: OpenAIUsage,

    /// System fingerprint (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

impl OpenAIChatResponse {
    /// Creates a new response.
    pub fn new(
        id: impl Into<String>,
        model: impl Into<String>,
        choices: Vec<OpenAIChoice>,
        usage: OpenAIUsage,
    ) -> Self {
        Self {
            id: id.into(),
            object: "chat.completion".to_string(),
            created: current_timestamp(),
            model: model.into(),
            choices,
            usage,
            system_fingerprint: None,
        }
    }
}

/// A single completion choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    /// Index of this choice in the list
    pub index: u32,

    /// Message content
    pub message: OpenAIMessage,

    /// Reason for finishing
    pub finish_reason: Option<String>,
}

impl OpenAIChoice {
    /// Creates a new choice.
    pub fn new(index: u32, message: OpenAIMessage, finish_reason: Option<&str>) -> Self {
        Self {
            index,
            message,
            finish_reason: finish_reason.map(String::from),
        }
    }
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAIUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,

    /// Number of tokens in the completion
    pub completion_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

impl OpenAIUsage {
    /// Creates new usage info.
    pub fn new(prompt_tokens: u32, completion_tokens: u32, total_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }
}

/// Error response matching OpenAI format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIErrorResponse {
    pub error: OpenAIError,
}

impl OpenAIErrorResponse {
    /// Creates a new error response.
    pub fn new(error_type: &str, message: impl Into<String>) -> Self {
        Self {
            error: OpenAIError {
                message: message.into(),
                r#type: error_type.to_string(),
                param: None,
                code: None,
            },
        }
    }
}

/// OpenAI error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIError {
    pub message: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Returns current Unix timestamp.
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
