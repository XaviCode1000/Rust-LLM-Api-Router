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

/// Pricing information for a model, in USD per 1 million tokens.
///
/// Aligns with industry standard: OpenAI, Anthropic, and LiteLLM
/// all publish prices as $/1M tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPricing {
    /// Cost per 1 million input (prompt) tokens in USD.
    pub input_cost_per_million_tokens: f64,
    /// Cost per 1 million output (completion) tokens in USD.
    pub output_cost_per_million_tokens: f64,
    /// ISO 8601 timestamp of when this pricing was last verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

impl ModelPricing {
    /// Creates a new `ModelPricing`.
    ///
    /// # Arguments
    /// * `input_cost_per_million_tokens` - Cost per 1M input tokens in USD
    /// * `output_cost_per_million_tokens` - Cost per 1M output tokens in USD
    pub fn new(input_cost_per_million_tokens: f64, output_cost_per_million_tokens: f64) -> Self {
        Self {
            input_cost_per_million_tokens,
            output_cost_per_million_tokens,
            last_updated: None,
        }
    }

    /// Sets the `last_updated` timestamp (ISO 8601).
    #[must_use]
    pub fn with_last_updated(mut self, timestamp: impl Into<String>) -> Self {
        self.last_updated = Some(timestamp.into());
        self
    }

    /// Estimates the cost for a request given approximate token counts.
    ///
    /// # Arguments
    /// * `input_tokens` - Estimated number of input tokens
    /// * `output_tokens` - Estimated number of output tokens
    ///
    /// # Returns
    /// Estimated cost in USD
    #[must_use]
    pub fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_cost_per_million_tokens;
        let output_cost =
            (output_tokens as f64 / 1_000_000.0) * self.output_cost_per_million_tokens;
        input_cost + output_cost
    }
}

/// Model information from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    /// Pricing information for cost-aware routing.
    /// `None` indicates pricing is unknown or not applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
}

impl Model {
    /// Creates a new `Model` without pricing information.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider_id: provider_id.into(),
            pricing: None,
        }
    }

    /// Creates a new `Model` with pricing information.
    pub fn with_pricing(
        id: impl Into<String>,
        name: impl Into<String>,
        provider_id: impl Into<String>,
        pricing: ModelPricing,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider_id: provider_id.into(),
            pricing: Some(pricing),
        }
    }
}
