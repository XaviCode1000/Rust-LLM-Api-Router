//! OpenAI provider implementation

use async_trait::async_trait;

use crate::domain::entities::{LlmRequest, LlmResponse};
use crate::domain::traits::LlmProvider;
use crate::error::Result;
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

    fn name(&self) -> &str {
        &self.name
    }
}
