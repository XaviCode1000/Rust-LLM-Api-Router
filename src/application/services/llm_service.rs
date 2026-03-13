use crate::domain::entities::{
    ChatRequest, ChatResponse, Model, Provider,
};
use crate::infrastructure::errors::AppError;
use crate::infrastructure::gateway::LlmGatewayImpl;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::stream::Stream;

/// Service trait for LLM operations
#[async_trait::async_trait]
pub trait LlmService: Send + Sync {
    /// Process a chat completion request (non-streaming)
    async fn chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, AppError>;

    /// Process a chat completion request (streaming)
    async fn stream_chat_completion(
        &self,
        request: ChatRequest,
        sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<Model>, AppError>;

    /// List configured providers
    async fn list_providers(&self) -> Result<Vec<Provider>, AppError>;

    /// Create a new provider configuration
    async fn create_provider(&self, provider: Provider) -> Result<Provider, AppError>;
}

/// Concrete implementation of LlmService
pub struct LlmServiceImpl {
    llm_gateway: Arc<LlmGatewayImpl>,
}

impl LlmServiceImpl {
    pub fn new(llm_gateway: Arc<LlmGatewayImpl>) -> Self {
        Self { llm_gateway }
    }
}

#[async_trait::async_trait]
impl LlmService for LlmServiceImpl {
    async fn chat_completion(
        &self,
        _request: ChatRequest,
    ) -> Result<ChatResponse, AppError> {
        // Stub implementation - returns a dummy response
        Ok(ChatResponse {
            id: "chatcmpl-stub".to_string(),
            choices: vec![crate::domain::entities::Choice {
                index: 0,
                message: crate::domain::entities::Message {
                    role: "assistant".to_string(),
                    content: "This is a stub response".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: crate::domain::entities::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        })
    }

    async fn stream_chat_completion(
        &self,
        _request: ChatRequest,
        _sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
        // Stub implementation - does nothing
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<Model>, AppError> {
        // Get API key from first active account (this is a simplified approach)
        // In a real implementation, we would need to get this from the account repository
        // For now, we'll return an empty list since we don't have access to the account repo here
        // A better approach would be to inject the account repository as well
        Ok(vec![])
    }

    async fn list_providers(&self) -> Result<Vec<Provider>, AppError> {
        // Stub implementation - returns empty list
        Ok(vec![])
    }

    async fn create_provider(&self, provider: Provider) -> Result<Provider, AppError> {
        // Stub implementation - returns the provider as-is
        Ok(provider)
    }
}