//! Groq provider implementation

use async_trait::async_trait;

use crate::domain::entities::{
    LlmRequest, LlmResponse, OpenAIChatRequest, OpenAIChatResponse, OpenAIMessage,
};
use crate::domain::traits::LlmProvider;
use crate::domain::Model;
use crate::error::{Error, Result};
use crate::infrastructure::http_client::SharedHttpClient;

pub struct GroqProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    api_url: String,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    http_client: SharedHttpClient,
}

impl GroqProvider {
    pub fn new(api_url: String, api_key: String, http_client: SharedHttpClient) -> Self {
        Self {
            name: "groq".to_string(),
            api_url,
            api_key,
            http_client,
        }
    }

    /// Make a chat completion request to Groq API
    /// Groq uses OpenAI-compatible API format
    pub async fn chat(&self, request: &OpenAIChatRequest) -> Result<OpenAIChatResponse> {
        let url = format!("{}/v1/chat/completions", self.api_url);

        let response = self
            .http_client
            .client()
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Failed to send Groq request: {}", e)))?;

        if response.status().is_success() {
            let chat_response: OpenAIChatResponse = response
                .json()
                .await
                .map_err(|e| Error::Internal(format!("Failed to parse Groq response: {}", e)))?;
            Ok(chat_response)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Internal(format!(
                "Groq API error ({}): {}",
                status, error_text
            )))
        }
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Convert LlmRequest to OpenAIChatRequest (Groq uses OpenAI-compatible format)
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

        // Use the typed chat method
        let openai_response = self.chat(&openai_request).await?;

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

    async fn list_models(&self, api_key: &str) -> Result<Vec<Model>> {
        let url = format!("{}/models", self.api_url);

        let response = self
            .http_client
            .client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Failed to fetch models from Groq: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Internal(format!(
                "Groq returned {}: {}",
                status, error_text
            )));
        }

        // Parse response - Groq uses OpenAI-compatible format: {"data": [{"id": "...", "object": "model", ...}]}
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse Groq models response: {}", e)))?;

        let mut models = Vec::new();
        let data_array = json
            .get("data")
            .and_then(|d: &serde_json::Value| d.as_array())
            .ok_or_else(|| Error::Internal("Invalid Groq models response format".to_string()))?;

        for item in data_array {
            if let Some(id) = item.get("id").and_then(|v: &serde_json::Value| v.as_str()) {
                let name = item
                    .get("owned_by")
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
    fn test_groq_provider_creation() {
        let client = Arc::new(HttpClient::new().unwrap());
        let provider = GroqProvider::new(
            "https://api.groq.com".to_string(),
            "sk-groq-key".to_string(),
            client,
        );

        assert_eq!(provider.name(), "groq");
    }
}
