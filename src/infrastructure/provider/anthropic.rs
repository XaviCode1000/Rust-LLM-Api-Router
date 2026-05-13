//! Anthropic provider implementation
//!
//! Anthropic uses a different API format than OpenAI.
//! This module handles the conversion between OpenAI-compatible types
//! and Anthropic's native format.
//!
//! See: <https://docs.anthropic.com/claude/reference/messages_post>

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{
    LlmRequest, LlmResponse, OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice, OpenAIMessage,
    OpenAIUsage,
};
use crate::domain::traits::{DomainResult, LlmProvider};
use crate::domain::Model;
use crate::error::{Error, Result};
use crate::infrastructure::http_client::SharedHttpClient;

/// Anthropic message request format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessageRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

/// Anthropic message format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessageResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<AnthropicContentBlock>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicContentBlock {
    r#type: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

pub struct AnthropicProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    api_url: String,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    http_client: SharedHttpClient,
}

impl AnthropicProvider {
    pub fn new(api_url: String, api_key: String, http_client: SharedHttpClient) -> Self {
        Self {
            name: "anthropic".to_string(),
            api_url,
            api_key,
            http_client,
        }
    }

    /// Make a chat completion request to Anthropic API
    ///
    /// Anthropic uses a different format than OpenAI:
    /// - Endpoint: /v1/messages (not /v1/chat/completions)
    /// - Headers: x-api-key and anthropic-version
    /// - Request format: { model, messages, max_tokens }
    /// - Response format: { id, content, usage, stop_reason }
    #[tracing::instrument(skip(self, request), fields(target = "anthropic", model = %request.model))]
    pub async fn chat(&self, request: &OpenAIChatRequest) -> Result<OpenAIChatResponse> {
        let url = format!("{}/v1/messages", self.api_url);

        // Extract system message if present
        let mut system_message: Option<String> = None;
        let mut messages: Vec<AnthropicMessage> = Vec::new();

        for msg in &request.messages {
            if msg.role == "system" {
                system_message = Some(msg.content.clone());
            } else {
                messages.push(AnthropicMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }

        // Build Anthropic request
        let anthropic_request = AnthropicMessageRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(1024),
            temperature: request.temperature,
            system: system_message,
        };

        let response = self
            .http_client
            .client()
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Failed to send Anthropic request: {}", e)))?;

        if response.status().is_success() {
            let anthropic_response: AnthropicMessageResponse =
                response.json().await.map_err(|e| {
                    Error::Internal(format!("Failed to parse Anthropic response: {}", e))
                })?;

            // Convert Anthropic response to OpenAI format
            let openai_response = OpenAIChatResponse {
                id: anthropic_response.id,
                object: "chat.completion".to_string(),
                created: crate::domain::entities::openai_types::current_timestamp(),
                model: anthropic_response.model,
                choices: vec![OpenAIChoice {
                    index: 0,
                    message: OpenAIMessage {
                        role: "assistant".to_string(),
                        content: anthropic_response
                            .content
                            .first()
                            .map(|c| c.text.clone())
                            .unwrap_or_default(),
                        name: None,
                    },
                    finish_reason: anthropic_response.stop_reason,
                }],
                usage: OpenAIUsage {
                    prompt_tokens: anthropic_response.usage.input_tokens,
                    completion_tokens: anthropic_response.usage.output_tokens,
                    total_tokens: anthropic_response.usage.input_tokens
                        + anthropic_response.usage.output_tokens,
                },
                system_fingerprint: None,
            };

            Ok(openai_response)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Internal(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )))
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, request: LlmRequest) -> DomainResult<LlmResponse> {
        // Convert LlmRequest to OpenAIChatRequest
        let openai_request = OpenAIChatRequest::new(
            request.model,
            request
                .messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role,
                    content: m.content,
                    name: None,
                })
                .collect(),
        );

        // Use the typed chat method, mapping errors to domain error
        let openai_response = self.chat(&openai_request).await.map_err(|e: Error| {
            crate::domain::errors::DomainError::ExternalServiceError(e.to_string())
        })?;

        // Convert OpenAIChatResponse to LlmResponse
        let response = LlmResponse {
            id: openai_response.id,
            choices: openai_response
                .choices
                .into_iter()
                .map(|c| crate::domain::Choice {
                    index: c.index,
                    message: crate::domain::Message {
                        role: c.message.role,
                        content: c.message.content,
                    },
                    finish_reason: c.finish_reason,
                })
                .collect(),
            usage: crate::domain::Usage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
        };

        Ok(response)
    }

    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>> {
        use crate::domain::errors::DomainError;

        let url = format!("{}/models", self.api_url);

        let response = self
            .http_client
            .client()
            .get(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalServiceError(format!(
                    "Failed to fetch models from Anthropic: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DomainError::ExternalServiceError(format!(
                "Anthropic returned {}: {}",
                status, error_text
            )));
        }

        // Parse response - Anthropic format: {"data": [{"id": "...", "type": "model", ...}]}
        let json: serde_json::Value = response.json().await.map_err(|e| {
            DomainError::Serialization(format!("Failed to parse Anthropic models response: {}", e))
        })?;

        let mut models = Vec::new();
        let data_array = json
            .get("data")
            .and_then(|d: &serde_json::Value| d.as_array())
            .ok_or_else(|| {
                DomainError::Serialization("Invalid Anthropic models response format".to_string())
            })?;

        for item in data_array {
            if let Some(id) = item.get("id").and_then(|v: &serde_json::Value| v.as_str()) {
                let name = item
                    .get("display_name")
                    .or_else(|| item.get("id"))
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or(id);

                models.push(Model::new(
                    id.to_string(),
                    name.to_string(),
                    self.name.clone(),
                ));
            }
        }

        Ok(models)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http_client::HttpClient;

    use std::sync::Arc;

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
}
