//! Application state

use std::collections::HashMap;
use std::sync::Arc;

use crate::app::router::llm_router::LlmRouter;
use crate::app::services::execution_plan::ExecutionPlannerConfig;
use crate::config::Settings;
use crate::domain::traits::AccountRepository;
use crate::infrastructure::gateway::llm_gateway::ProviderConfig;
use crate::infrastructure::{HttpClient, JsonAccountRepository, LlmGatewayImpl, Metrics};

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
    pub account_repo: Arc<dyn AccountRepository>,
    pub llm_gateway: Arc<LlmGatewayImpl>,
    pub provider_config: Arc<HashMap<String, ProviderConfig>>,
    /// LLM Router with ExecutionPlanner for intelligent request routing
    pub llm_router: Arc<LlmRouter<dyn AccountRepository>>,
}

impl AppState {
    pub fn new(config: Settings) -> Result<Self, crate::Error> {
        let http_client = Arc::new(HttpClient::new()?);
        let metrics = Arc::new(Metrics::new()?);
        let account_repo: Arc<dyn AccountRepository> = Arc::new(
            JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?,
        );

        // Create LLM Gateway with default providers and 1 hour cache TTL
        let provider_config =
            Arc::new(crate::infrastructure::gateway::llm_gateway::default_providers());
        let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
            http_client.clone(),
            account_repo.clone(),
            (*provider_config).clone(),
            3600, // 1 hour cache TTL
        ));

        // Create LLM Router with ExecutionPlanner
        let planner_config = ExecutionPlannerConfig::default();
        let llm_router = Arc::new(LlmRouter::new(
            http_client.clone(),
            account_repo.clone(),
            provider_config.clone(),
            planner_config,
        ));

        Ok(Self {
            config,
            http_client,
            metrics,
            account_repo,
            llm_gateway,
            provider_config,
            llm_router,
        })
    }

    /// Create AppState with custom provider config (for testing)
    pub fn with_provider_config(
        config: Settings,
        http_client: Arc<HttpClient>,
        account_repo: Arc<dyn AccountRepository>,
        provider_config: HashMap<String, ProviderConfig>,
    ) -> Result<Self, crate::Error> {
        let metrics = Arc::new(Metrics::new()?);
        let provider_config_arc = Arc::new(provider_config.clone());
        let llm_gateway = Arc::new(LlmGatewayImpl::with_config(
            http_client.clone(),
            account_repo.clone(),
            provider_config.clone(),
            3600, // 1 hour cache TTL
        ));

        // Create LLM Router with ExecutionPlanner for testing
        let planner_config = ExecutionPlannerConfig::default();
        let llm_router = Arc::new(LlmRouter::new(
            http_client.clone(),
            account_repo.clone(),
            provider_config_arc.clone(),
            planner_config,
        ));

        Ok(Self {
            config,
            http_client,
            metrics,
            account_repo,
            llm_gateway,
            provider_config: provider_config_arc,
            llm_router,
        })
    }
}
