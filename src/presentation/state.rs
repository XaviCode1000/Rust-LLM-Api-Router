//! Application state

use std::sync::Arc;

use crate::config::Settings;
use crate::infrastructure::{HttpClient, Metrics};

#[derive(Clone)]
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub fn new(config: Settings) -> Result<Self, crate::Error> {
        let http_client = Arc::new(HttpClient::new()?);
        let metrics = Arc::new(Metrics::new()?);

        Ok(Self {
            config,
            http_client,
            metrics,
        })
    }
}
