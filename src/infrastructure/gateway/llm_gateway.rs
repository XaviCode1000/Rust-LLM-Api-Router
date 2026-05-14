//! LLM Gateway implementation - aggregates multiple providers
//!
//! This module provides the concrete implementation of the LlmGateway trait,
//! handling communication with multiple LLM providers and aggregating results.
//!
//! ## Design
//!
//! Supports dependency injection via `ProviderConfig` for testability:
//!
//! ```text
//! // Production usage (backward compatible)
//! // let gateway = LlmGatewayImpl::new(http_client, account_repo, 3600);
//!
//! // Test usage (custom config)
//! // let config = ProviderConfig::builder()
//! //     .with_provider("openai", "https://api.openai.com/v1", "sk-test")
//! //     .build();
//! // let gateway = LlmGatewayImpl::with_config(http_client, account_repo, config, 3600);
//! ```

use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::entities::{ChatRequest, ChatResponse, Model};
use crate::domain::errors::DomainError;
use crate::domain::traits::DomainResult;
use crate::domain::traits::{AccountRepository, LlmGateway};
use crate::infrastructure::http_client::SharedHttpClient;

/// Configuration for a single provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub models_endpoint: String,
}

impl ProviderConfig {
    /// Create a new provider config
    pub fn new(id: &str, name: &str, base_url: &str, models_endpoint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            models_endpoint: models_endpoint.to_string(),
        }
    }

    /// Create a builder for provider config
    pub fn builder() -> ProviderConfigBuilder {
        ProviderConfigBuilder::new()
    }

    /// Create default config with URL (for backward compatibility)
    pub fn default_with_url(base_url: &str, _api_key: &str) -> HashMap<String, Self> {
        let mut providers = HashMap::new();
        providers.insert(
            "default".to_string(),
            Self::new("default", "Default", base_url, "/models"),
        );
        providers
    }
}

/// Builder for ProviderConfig HashMap
#[derive(Debug, Default)]
pub struct ProviderConfigBuilder {
    providers: HashMap<String, ProviderConfig>,
}

impl ProviderConfigBuilder {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a provider to the builder
    pub fn with_provider(
        mut self,
        id: &str,
        name: &str,
        base_url: &str,
        models_endpoint: &str,
    ) -> Self {
        let config = ProviderConfig::new(id, name, base_url, models_endpoint);
        self.providers.insert(id.to_string(), config);
        self
    }

    /// Add a provider with simplified signature (id, base_url, api_key)
    pub fn with_provider_simple(mut self, id: &str, base_url: &str, _api_key: &str) -> Self {
        let config = ProviderConfig::new(id, id, base_url, "/models");
        self.providers.insert(id.to_string(), config);
        self
    }

    /// Build the provider config HashMap
    pub fn build(self) -> HashMap<String, ProviderConfig> {
        self.providers
    }
}

/// Default provider configurations — mirrors LiteLLM's OpenAI-compatible providers
pub fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();

    let entries: &[(&str, &str, &str, &str)] = &[
        // Major providers
        ("openai", "OpenAI", "https://api.openai.com/v1", "/models"),
        (
            "anthropic",
            "Anthropic",
            "https://api.anthropic.com/v1",
            "/models",
        ),
        ("groq", "Groq", "https://api.groq.com/openai/v1", "/models"),
        // OpenAI-compatible cloud providers
        (
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/v1",
            "/models",
        ),
        (
            "together",
            "Together AI",
            "https://api.together.xyz/v1",
            "/models",
        ),
        (
            "fireworks",
            "Fireworks AI",
            "https://api.fireworks.ai/inference/v1",
            "/models",
        ),
        ("xai", "xAI (Grok)", "https://api.x.ai/v1", "/models"),
        (
            "perplexity",
            "Perplexity",
            "https://api.perplexity.ai/v1",
            "/models",
        ),
        (
            "openrouter",
            "OpenRouter",
            "https://openrouter.ai/api/v1",
            "/models",
        ),
        (
            "mistral",
            "Mistral AI",
            "https://api.mistral.ai/v1",
            "/models",
        ),
        (
            "cerebras",
            "Cerebras",
            "https://api.cerebras.ai/v1",
            "/models",
        ),
        (
            "cloudflare",
            "Cloudflare AI Gateway",
            "https://gateway.ai.cloudflare.com/v1",
            "/models",
        ),
        // Local inference servers
        ("ollama", "Ollama", "http://localhost:11434/v1", "/models"),
        (
            "lmstudio",
            "LM Studio",
            "http://localhost:1234/v1",
            "/models",
        ),
        ("vllm", "vLLM", "http://localhost:8000/v1", "/models"),
        // Platform / specialized providers
        (
            "replicate",
            "Replicate",
            "https://api.replicate.com/v1",
            "/models",
        ),
        (
            "huggingface",
            "HuggingFace",
            "https://api-inference.huggingface.co",
            "/models",
        ),
        (
            "anyscale",
            "Anyscale",
            "https://api.endpoints.anyscale.com/v1",
            "/models",
        ),
        (
            "deepinfra",
            "DeepInfra",
            "https://api.deepinfra.com/v1",
            "/models",
        ),
        ("novita", "Novita AI", "https://api.novita.ai/v1", "/models"),
        (
            "sambanova",
            "SambaNova",
            "https://api.sambanova.ai/v1",
            "/models",
        ),
        // Cloud hyperscaler services
        (
            "azure",
            "Azure OpenAI",
            "https://{resource}.openai.azure.com/v1",
            "/models",
        ),
        (
            "bedrock",
            "AWS Bedrock",
            "https://bedrock-runtime.{region}.amazonaws.com",
            "/models",
        ),
        (
            "vertexai",
            "Google Vertex AI",
            "https://{region}-aiplatform.googleapis.com/v1",
            "/models",
        ),
        // Additional model providers
        ("cohere", "Cohere", "https://api.cohere.ai/v1", "/models"),
        ("ai21", "AI21 Labs", "https://api.ai21.com/v1", "/models"),
        (
            "aleph_alpha",
            "Aleph Alpha",
            "https://api.aleph-alpha.com/v1",
            "/models",
        ),
        (
            "nvidia",
            "NVIDIA NIM",
            "https://integrate.api.nvidia.com/v1",
            "/models",
        ),
        (
            "google",
            "Google AI Studio",
            "https://generativelanguage.googleapis.com/v1",
            "/models",
        ),
        // Free-tier providers (from awesome-free-llm-apis)
        (
            "zhipu",
            "Zhipu AI",
            "https://open.bigmodel.cn/api/paas/v4",
            "/models",
        ),
        (
            "github",
            "GitHub Models",
            "https://models.inference.ai.azure.com",
            "/models",
        ),
        (
            "kluster",
            "Kluster AI",
            "https://api.kluster.ai/v1",
            "/models",
        ),
        ("llm7", "LLM7.io", "https://api.llm7.io/v1", "/models"),
        (
            "siliconflow",
            "SiliconFlow",
            "https://api.siliconflow.cn/v1",
            "/models",
        ),
    ];

    for &(id, name, base_url, models_endpoint) in entries {
        providers.insert(
            id.to_string(),
            ProviderConfig::new(id, name, base_url, models_endpoint),
        );
    }

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

/// Type alias for cache entry data (models + timestamp)
type CacheEntry = (Vec<Model>, chrono::DateTime<chrono::Utc>);

/// Type alias for the models cache with TTL
type ModelsCache = RwLock<HashMap<String, CacheEntry>>;

impl LlmGatewayImpl {
    /// Create a new LLM gateway with default providers (backward compatible)
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

    /// Create a new LLM gateway with custom provider config (for testing)
    pub fn with_config(
        http_client: SharedHttpClient,
        account_repo: Arc<dyn AccountRepository>,
        config: HashMap<String, ProviderConfig>,
        cache_ttl_seconds: u64,
    ) -> Self {
        Self {
            http_client,
            account_repo,
            providers: config,
            models_cache: RwLock::new(HashMap::new()),
            cache_ttl_seconds,
        }
    }

    /// Get the configured providers (for testing)
    pub fn providers(&self) -> &HashMap<String, ProviderConfig> {
        &self.providers
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

        let url = format!(
            "{}{}",
            self.http_client
                .mock_base_url()
                .map(|url| url.to_string())
                .unwrap_or_else(|| config.base_url.clone()),
            config.models_endpoint
        );

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
    pub fn is_cache_valid(cached_at: &chrono::DateTime<chrono::Utc>, ttl_seconds: u64) -> bool {
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
            .map(|a| a.provider_id.clone())
            .collect::<std::collections::HashSet<_>>();

        if enabled_providers.is_empty() {
            return Ok(vec![]);
        }

        // Fetch models from all providers in parallel
        let futures: Vec<_> = enabled_providers
            .iter()
            .map(|provider_id| self.fetch_provider_models(provider_id.as_str(), api_key))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http_client::HttpClient;
    use crate::infrastructure::persistence::json_account_repository::JsonAccountRepository;
    use tempfile::TempDir;

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::builder()
            .with_provider("openai", "OpenAI", "https://api.openai.com/v1", "/models")
            .with_provider("groq", "Groq", "https://api.groq.com/openai/v1", "/models")
            .build();

        assert_eq!(config.len(), 2);
        assert!(config.contains_key("openai"));
        assert!(config.contains_key("groq"));
    }

    #[test]
    fn test_provider_config_builder_simple() {
        let config = ProviderConfig::builder()
            .with_provider_simple("openai", "https://api.openai.com/v1", "sk-test")
            .with_provider_simple("groq", "https://api.groq.com/openai/v1", "sk-groq")
            .build();

        assert_eq!(config.len(), 2);
        let openai_config = config.get("openai").unwrap();
        assert_eq!(openai_config.base_url, "https://api.openai.com/v1");
        assert_eq!(openai_config.models_endpoint, "/models");
    }

    #[test]
    fn test_provider_config_default_with_url() {
        let config = ProviderConfig::default_with_url("https://custom.api.com/v1", "sk-custom");

        assert_eq!(config.len(), 1);
        assert!(config.contains_key("default"));
        let default_config = config.get("default").unwrap();
        assert_eq!(default_config.base_url, "https://custom.api.com/v1");
    }

    #[tokio::test]
    async fn test_gateway_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
        let http_client = Arc::new(HttpClient::new().unwrap());

        let config = ProviderConfig::builder()
            .with_provider(
                "test-provider",
                "Test",
                "https://test.api.com/v1",
                "/models",
            )
            .build();

        let gateway = LlmGatewayImpl::with_config(http_client, repo, config, 3600);

        // Verify custom config was set
        assert_eq!(gateway.providers().len(), 1);
        assert!(gateway.providers().contains_key("test-provider"));
    }

    #[tokio::test]
    async fn test_gateway_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
        let http_client = Arc::new(HttpClient::new().unwrap());

        let gateway = LlmGatewayImpl::new(http_client, repo, 3600);

        // Verify default providers are loaded
        assert!(gateway.providers().len() >= 25);
        assert!(gateway.providers().contains_key("openai"));
        assert!(gateway.providers().contains_key("groq"));
    }

    #[test]
    fn test_is_cache_valid() {
        let now = chrono::Utc::now();
        let ttl = 3600;

        // Fresh cache entry
        assert!(LlmGatewayImpl::is_cache_valid(&now, ttl));

        // Old cache entry (2 hours ago)
        let old = now - chrono::Duration::hours(2);
        assert!(!LlmGatewayImpl::is_cache_valid(&old, ttl));

        // Edge case: exactly at TTL
        let edge = now - chrono::Duration::seconds(ttl as i64);
        assert!(!LlmGatewayImpl::is_cache_valid(&edge, ttl));

        // Just before TTL
        let just_before = now - chrono::Duration::seconds((ttl - 1) as i64);
        assert!(LlmGatewayImpl::is_cache_valid(&just_before, ttl));
    }
}
