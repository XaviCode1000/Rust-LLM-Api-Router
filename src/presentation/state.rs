//! Application state

use std::sync::Arc;

use crate::config::Settings;
use crate::domain::traits::AccountRepository;
use crate::infrastructure::{HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics};

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
    pub account_repo: Arc<dyn AccountRepository>,
    pub llm_gateway: Arc<LlmGatewayImpl>,
}

impl AppState {
    pub fn new(config: Settings) -> Result<Self, crate::Error> {
        let http_client = Arc::new(HttpClient::new()?);
        let metrics = Arc::new(Metrics::new()?);
        let account_repo: Arc<dyn AccountRepository> = Arc::new(
            JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?,
        );

        // Create LLM Gateway with 1 hour cache TTL
        let llm_gateway = Arc::new(LlmGatewayImpl::new(
            http_client.clone(),
            account_repo.clone(),
            3600, // 1 hour cache TTL
        ));

        Ok(Self {
            config,
            http_client,
            metrics,
            account_repo,
            llm_gateway,
        })
    }
}
