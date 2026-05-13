//! Typed request body for forwarding to LLM providers
//!
//! This module provides zero-allocation borrowed structs for building
//! the JSON request body sent to external LLM providers. Using typed
//! structs with `serde_json::to_value()` replaces the `serde_json::json!()`
//! macro in hot paths, eliminating unnecessary intermediate allocations
//! and providing compile-time type safety.
//!
//! # Performance
//!
//! The borrowed variants (`&'a str`) avoid cloning strings from the
//! domain `Message` type when serializing. This matters because this
//! code runs on EVERY request forwarded to a provider.

use serde::Serialize;

/// Request body for LLM provider chat completions.
///
/// Uses borrowed references to avoid cloning strings from domain types
/// during serialization. The wire format matches the OpenAI Chat
/// Completions API specification.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderChatRequest<'a> {
    /// Model identifier (e.g., "gpt-4", "claude-3-sonnet")
    model: &'a str,

    /// Conversation messages
    messages: &'a [ProviderChatMessage<'a>],

    /// Sampling temperature (0.0 to 2.0). Defaults to 0.7 if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    /// Maximum tokens to generate. Defaults to 1024 if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,

    /// Whether to stream the response. Defaults to false if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl<'a> ProviderChatRequest<'a> {
    /// Creates a new provider chat request builder.
    #[must_use]
    pub fn builder(model: &'a str, messages: &'a [ProviderChatMessage<'a>]) -> Self {
        Self {
            model,
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
        }
    }

    /// Sets the temperature parameter.
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the max_tokens parameter.
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the stream parameter.
    #[must_use]
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Applies defaults for missing optional parameters.
    ///
    /// This ensures the wire format matches the previous `json!()` behavior
    /// where defaults were always included. However, with `skip_serializing_if`,
    /// we only serialize when values differ from defaults, reducing payload size.
    #[must_use]
    pub fn with_defaults(self) -> Self {
        Self {
            temperature: self.temperature.or(Some(0.7)),
            max_tokens: self.max_tokens.or(Some(1024)),
            stream: self.stream.or(Some(false)),
            ..self
        }
    }
}

/// A single message in a chat conversation for provider requests.
///
/// Borrowed to avoid cloning from the domain `Message` type.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderChatMessage<'a> {
    /// Role of the message sender ("system", "user", "assistant")
    role: &'a str,

    /// Content of the message
    content: &'a str,
}

impl<'a> ProviderChatMessage<'a> {
    /// Creates a new provider chat message from borrowed strings.
    #[must_use]
    pub fn new(role: &'a str, content: &'a str) -> Self {
        Self { role, content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_matches_json_macro_format() {
        let messages = vec![
            ProviderChatMessage::new("system", "You are a helpful assistant."),
            ProviderChatMessage::new("user", "Hello, world!"),
        ];

        let body = ProviderChatRequest::builder("gpt-4", &messages)
            .with_temperature(0.7)
            .with_max_tokens(1024)
            .with_stream(false);

        let value = serde_json::to_value(&body).unwrap();

        assert_eq!(value["model"], "gpt-4");
        assert_eq!(value["messages"].as_array().unwrap().len(), 2);
        assert_eq!(value["messages"][0]["role"], "system");
        assert_eq!(
            value["messages"][0]["content"],
            "You are a helpful assistant."
        );
        assert_eq!(value["messages"][1]["role"], "user");
        assert_eq!(value["messages"][1]["content"], "Hello, world!");
        assert!(
            (value["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001,
            "temperature should be ~0.7"
        );
        assert_eq!(value["max_tokens"], 1024);
        assert_eq!(value["stream"], false);
    }

    #[test]
    fn test_skip_serializing_if_none() {
        let messages = vec![ProviderChatMessage::new("user", "test")];

        let body = ProviderChatRequest::builder("gpt-4", &messages);

        let value = serde_json::to_value(&body).unwrap();

        // Optional fields should be absent when None
        assert!(value.get("temperature").is_none());
        assert!(value.get("max_tokens").is_none());
        assert!(value.get("stream").is_none());
        assert_eq!(value["model"], "gpt-4");
    }

    #[test]
    fn test_with_defaults_applies_correct_values() {
        let messages = vec![ProviderChatMessage::new("user", "test")];

        let body = ProviderChatRequest::builder("gpt-4", &messages).with_defaults();

        let value = serde_json::to_value(&body).unwrap();

        assert!(
            (value["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001,
            "temperature should be ~0.7"
        );
        assert_eq!(value["max_tokens"], 1024);
        assert_eq!(value["stream"], false);
    }

    #[test]
    fn test_empty_messages_array() {
        let messages: Vec<ProviderChatMessage> = vec![];
        let body = ProviderChatRequest::builder("gpt-4", &messages).with_defaults();

        let value = serde_json::to_value(&body).unwrap();
        assert!(value["messages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_message_from_domain_type() {
        // Verify we can construct from domain Message without cloning
        let domain_msg = crate::domain::entities::Message::user("Hello");
        let provider_msg = ProviderChatMessage::new(&domain_msg.role, &domain_msg.content);

        let messages = vec![provider_msg];
        let body = ProviderChatRequest::builder("gpt-4", &messages);

        let value = serde_json::to_value(&body).unwrap();
        assert_eq!(value["messages"][0]["role"], "user");
        assert_eq!(value["messages"][0]["content"], "Hello");
    }
}
