//! OpenAI provider implementation

use async_trait::async_trait;

use crate::domain::entities::{LlmRequest, LlmResponse};
use crate::domain::traits::LlmProvider;
use crate::domain::Model;
use crate::error::{Error, Result};
use crate::infrastructure::http_client::SharedHttpClient;

pub struct OpenAiProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    api_url: String,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    http_client: SharedHttpClient,
}

impl OpenAiProvider {
    pub fn new(api_url: String, api_key: String, http_client: SharedHttpClient) -> Self {
        Self {
            name: "openai".to_string(),
            api_url,
            api_key,
            http_client,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, _request: LlmRequest) -> Result<LlmResponse> {
        // Implementation would go here
        todo!("Implement OpenAI chat completion")
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
            .map_err(|e| {
                Error::Internal(format!("Failed to fetch models from OpenAI: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Internal(format!(
                "OpenAI returned {}: {}",
                status, error_text
            )));
        }

        // Parse response - OpenAI format: {"data": [{"id": "...", "object": "model", ...}]}
        let json: serde_json::Value = response.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse OpenAI models response: {}", e))
        })?;

        let mut models = Vec::new();
        let data_array = json
            .get("data")
            .and_then(|d: &serde_json::Value| d.as_array())
            .ok_or_else(|| Error::Internal("Invalid OpenAI models response format".to_string()))?;

        for item in data_array {
            if let Some(id) = item.get("id").and_then(|v: &serde_json::Value| v.as_str()) {
                let name = item
                    .get("owned_by")
                    .or_else(|| item.get("id"))
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or(id);

                models.push(Model::new(id.to_string(), name.to_string(), self.name.clone()));
            }
        }

        Ok(models)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
