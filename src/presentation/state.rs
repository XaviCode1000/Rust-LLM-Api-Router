//! Application state

use std::sync::Arc;

use crate::config::Settings;
use crate::infrastructure::{HttpClient, JsonAccountRepository, Metrics};

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
    pub account_repo: Arc<JsonAccountRepository>,
}

impl AppState {
    pub fn new(config: Settings) -> Result<Self, crate::Error> {
        let http_client = Arc::new(HttpClient::new()?);
        let metrics = Arc::new(Metrics::new()?);
        let account_repo = Arc::new(JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?);

        Ok(Self {
            config,
            http_client,
            metrics,
            account_repo,
        })
    }
}
