//! LLM Gateway implementation - aggregates multiple providers
//!
//! This module provides the concrete implementation of the LlmGateway trait,
//! handling communication with multiple LLM providers and aggregating results.

use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::entities::{ChatRequest, ChatResponse, Model};
use crate::domain::errors::DomainError;
use crate::domain::errors::DomainResult;
use crate::domain::traits::{AccountRepository, LlmGateway};
use crate::infrastructure::http_client::SharedHttpClient;

/// Type alias for cache entry data (models + timestamp)
type CacheEntry = (Vec<Model>, chrono::DateTime<chrono::Utc>);

/// Type alias for the models cache with TTL
type ModelsCache = RwLock<HashMap<String, CacheEntry>>;

/// Configuration for a single provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub models_endpoint: String,
}

impl ProviderConfig {
    pub fn new(id: &str, name: &str, base_url: &str, models_endpoint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            models_endpoint: models_endpoint.to_string(),
        }
    }
}

/// Default provider configurations
pub fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();

    providers.insert(
        "openai".to_string(),
        ProviderConfig::new("openai", "OpenAI", "https://api.openai.com/v1", "/models"),
    );

    providers.insert(
        "groq".to_string(),
        ProviderConfig::new("groq", "Groq", "https://api.groq.com/openai/v1", "/models"),
    );

    providers.insert(
        "openrouter".to_string(),
        ProviderConfig::new(
            "openrouter",
            "OpenRouter",
            "https://openrouter.ai/api/v1",
            "/models",
        ),
    );

    providers.insert(
        "mistral".to_string(),
        ProviderConfig::new(
            "mistral",
            "Mistral AI",
            "https://api.mistral.ai/v1",
            "/models",
        ),
    );

    providers.insert(
        "cerebras".to_string(),
        ProviderConfig::new(
            "cerebras",
            "Cerebras",
            "https://api.cerebras.ai/v1",
            "/models",
        ),
    );

    providers.insert(
        "anthropic".to_string(),
        ProviderConfig::new(
            "anthropic",
            "Anthropic",
            "https://api.anthropic.com/v1",
            "/models",
        ),
    );

    providers
}

/// Concrete LLM Gateway implementation
pub struct LlmGatewayImpl {
    http_client: SharedHttpClient,
    account_repo: Arc<dyn AccountRepository>,
    providers: HashMap<String, ProviderConfig>,
    /// Cache for models with TTL
    models_cache: ModelsCache,
    cache_ttl_seconds: u64,
}

impl LlmGatewayImpl {
    pub fn new(
        http_client: SharedHttpClient,
        account_repo: Arc<dyn AccountRepository>,
        cache_ttl_seconds: u64,
    ) -> Self {
        Self {
            http_client,
            account_repo,
            providers: default_providers(),
            models_cache: RwLock::new(HashMap::new()),
            cache_ttl_seconds,
        }
    }

    /// Fetch models from a single provider
    async fn fetch_provider_models(
        &self,
        provider_id: &str,
        api_key: &str,
    ) -> DomainResult<Vec<Model>> {
        let config = self.providers.get(provider_id).ok_or_else(|| {
            DomainError::ProviderNotFound(format!("Provider '{}' not configured", provider_id))
        })?;

        let url = format!("{}{}", config.base_url, config.models_endpoint);

        let response = self
            .http_client
            .client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| {
                DomainError::ExternalServiceError(format!(
                    "Failed to fetch models from {}: {}",
                    provider_id, e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DomainError::ExternalServiceError(format!(
                "Provider {} returned {}: {}",
                provider_id, status, error_text
            )));
        }

        // Parse response - handle different provider formats
        let models = self.parse_models_response(provider_id, response).await?;
        Ok(models)
    }

    /// Parse models response based on provider type
    async fn parse_models_response(
        &self,
        provider_id: &str,
        response: reqwest::Response,
    ) -> DomainResult<Vec<Model>> {
        let json: serde_json::Value = response.json().await.map_err(|e| {
            DomainError::Serialization(format!(
                "Failed to parse models response from {}: {}",
                provider_id, e
            ))
        })?;

        // Most providers use OpenAI-compatible format: {"data": [{"id": "...", ...}]}
        let data_array = json
            .get("data")
            .and_then(|d: &serde_json::Value| d.as_array())
            .ok_or_else(|| {
                DomainError::Serialization(format!(
                    "Invalid models response format from {}",
                    provider_id
                ))
            })?;

        let mut models = Vec::new();
        for item in data_array {
            if let Some(id) = item.get("id").and_then(|v: &serde_json::Value| v.as_str()) {
                let name = item
                    .get("name")
                    .or_else(|| item.get("owned_by"))
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or(id);

                models.push(Model::new(id, name, provider_id));
            }
        }

        Ok(models)
    }

    /// Check if cache is valid
    fn is_cache_valid(cached_at: &chrono::DateTime<chrono::Utc>, ttl_seconds: u64) -> bool {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(*cached_at).num_seconds() as u64;
        elapsed < ttl_seconds
    }
}

#[async_trait]
impl LlmGateway for LlmGatewayImpl {
    async fn chat(&self, _request: ChatRequest, _api_key: &str) -> DomainResult<ChatResponse> {
        // Delegated to specific provider implementations
        // This is handled by the service layer
        Err(DomainError::NotImplemented(
            "Direct chat via gateway not implemented, use LlmService".to_string(),
        ))
    }

    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>> {
        // Check cache first
        {
            let cache = self.models_cache.read().await;
            if let Some((models, cached_at)) = cache.get("all") {
                if Self::is_cache_valid(cached_at, self.cache_ttl_seconds) {
                    return Ok(models.clone());
                }
            }
        }

        // Get all enabled providers
        let enabled_providers = self
            .account_repo
            .find_all()
            .await?
            .into_iter()
            .filter(|a| a.is_active)
            .map(|a| a.provider_id)
            .collect::<std::collections::HashSet<_>>();

        if enabled_providers.is_empty() {
            return Ok(vec![]);
        }

        // Fetch models from all providers in parallel
        let futures: Vec<_> = enabled_providers
            .iter()
            .map(|provider_id| self.fetch_provider_models(provider_id, api_key))
            .collect();

        let results = join_all(futures).await;

        // Aggregate results, ignoring failed providers (resilience pattern)
        let mut all_models = Vec::new();
        for result in results {
            match result {
                Ok(models) => all_models.extend(models),
                Err(e) => {
                    // Log error but continue with other providers
                    tracing::warn!("Failed to fetch models from provider: {}", e);
                }
            }
        }

        // Update cache
        {
            let mut cache = self.models_cache.write().await;
            cache.insert("all".to_string(), (all_models.clone(), chrono::Utc::now()));
        }

        Ok(all_models)
    }
}
