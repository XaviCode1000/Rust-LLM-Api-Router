//! Application settings loaded from environment

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub app_host: String,
    pub app_port: u16,
    pub log_level: String,
    pub providers: Vec<ProviderConfig>,
    pub cascading_min_quality_score: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub api_url: String,
    pub api_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_host: "0.0.0.0".to_string(),
            app_port: 8080,
            log_level: "info".to_string(),
            providers: Vec::new(),
            cascading_min_quality_score: 0.75,
        }
    }
}
