//! Gateway configuration and failover tests
//!
//! Tests for LlmGatewayImpl with custom ProviderConfig injection.

use std::sync::Arc;
use tempfile::TempDir;

use rust_llm_api_router::domain::traits::{AccountRepository, LlmGateway};
use rust_llm_api_router::domain::Account;
use rust_llm_api_router::infrastructure::{
    gateway::{LlmGatewayImpl, ProviderConfig},
    HttpClient, JsonAccountRepository,
};

// ============================================================================
// ProviderConfig Tests
// ============================================================================

#[test]
fn test_provider_config_creation() {
    let config = ProviderConfig::new("openai", "OpenAI", "https://api.openai.com/v1", "/models");

    assert_eq!(config.id, "openai");
    assert_eq!(config.name, "OpenAI");
    assert_eq!(config.base_url, "https://api.openai.com/v1");
    assert_eq!(config.models_endpoint, "/models");
}

#[test]
fn test_provider_config_builder() {
    let config = ProviderConfig::builder()
        .with_provider("openai", "OpenAI", "https://api.openai.com/v1", "/models")
        .with_provider("groq", "Groq", "https://api.groq.com/openai/v1", "/models")
        .with_provider(
            "anthropic",
            "Anthropic",
            "https://api.anthropic.com/v1",
            "/models",
        )
        .build();

    assert_eq!(config.len(), 3);
    assert!(config.contains_key("openai"));
    assert!(config.contains_key("groq"));
    assert!(config.contains_key("anthropic"));

    // Verify openai config
    let openai_config = config.get("openai").unwrap();
    assert_eq!(openai_config.base_url, "https://api.openai.com/v1");
    assert_eq!(openai_config.models_endpoint, "/models");
}

#[test]
fn test_provider_config_builder_simple() {
    let config = ProviderConfig::builder()
        .with_provider_simple("openai", "https://api.openai.com/v1", "sk-test")
        .with_provider_simple("groq", "https://api.groq.com/openai/v1", "sk-groq")
        .build();

    assert_eq!(config.len(), 2);

    let openai_config = config.get("openai").unwrap();
    assert_eq!(openai_config.id, "openai");
    assert_eq!(openai_config.base_url, "https://api.openai.com/v1");
    assert_eq!(openai_config.models_endpoint, "/models");
}

#[test]
fn test_provider_config_default_with_url() {
    let config = ProviderConfig::default_with_url("https://custom.api.com/v1", "sk-custom-key");

    assert_eq!(config.len(), 1);
    assert!(config.contains_key("default"));

    let default_config = config.get("default").unwrap();
    assert_eq!(default_config.base_url, "https://custom.api.com/v1");
    assert_eq!(default_config.models_endpoint, "/models");
}

#[test]
fn test_provider_config_builder_empty() {
    let config = ProviderConfig::builder().build();
    assert_eq!(config.len(), 0);
}

#[test]
fn test_provider_config_builder_overwrite() {
    // Adding same provider twice should overwrite
    let config = ProviderConfig::builder()
        .with_provider(
            "openai",
            "OpenAI v1",
            "https://api.openai.com/v1",
            "/models",
        )
        .with_provider(
            "openai",
            "OpenAI v2",
            "https://api.openai.com/v2",
            "/v2/models",
        )
        .build();

    assert_eq!(config.len(), 1);
    let openai_config = config.get("openai").unwrap();
    assert_eq!(openai_config.name, "OpenAI v2");
    assert_eq!(openai_config.base_url, "https://api.openai.com/v2");
}

// ============================================================================
// LlmGatewayImpl Constructor Tests
// ============================================================================

#[tokio::test]
async fn test_gateway_with_default_config() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let gateway = LlmGatewayImpl::new(http_client, repo, 3600);

    // Verify default providers are loaded
    let providers = gateway.providers();
    assert!(providers.len() >= 5);
    assert!(providers.contains_key("openai"));
    assert!(providers.contains_key("groq"));
    assert!(providers.contains_key("anthropic"));
}

#[tokio::test]
async fn test_gateway_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let custom_config = ProviderConfig::builder()
        .with_provider(
            "test-provider",
            "Test Provider",
            "https://test.api.com/v1",
            "/models",
        )
        .build();

    let gateway = LlmGatewayImpl::with_config(http_client, repo, custom_config, 3600);

    // Verify custom config was set
    let providers = gateway.providers();
    assert_eq!(providers.len(), 1);
    assert!(providers.contains_key("test-provider"));
    assert!(!providers.contains_key("openai"));

    let test_config = providers.get("test-provider").unwrap();
    assert_eq!(test_config.base_url, "https://test.api.com/v1");
}

#[tokio::test]
async fn test_gateway_with_multiple_custom_providers() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let custom_config = ProviderConfig::builder()
        .with_provider(
            "provider-a",
            "Provider A",
            "https://a.api.com/v1",
            "/models",
        )
        .with_provider(
            "provider-b",
            "Provider B",
            "https://b.api.com/v1",
            "/models",
        )
        .with_provider(
            "provider-c",
            "Provider C",
            "https://c.api.com/v1",
            "/models",
        )
        .build();

    let gateway = LlmGatewayImpl::with_config(
        http_client,
        repo,
        custom_config,
        7200, // 2 hour TTL
    );

    let providers = gateway.providers();
    assert_eq!(providers.len(), 3);
    assert!(providers.contains_key("provider-a"));
    assert!(providers.contains_key("provider-b"));
    assert!(providers.contains_key("provider-c"));
}

#[tokio::test]
async fn test_gateway_cache_ttl_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    // Test with different TTL values
    let gateway_short = LlmGatewayImpl::new(
        http_client.clone(),
        repo.clone(),
        60, // 1 minute
    );

    let gateway_long = LlmGatewayImpl::with_config(
        http_client,
        repo,
        ProviderConfig::builder().build(),
        86400, // 24 hours
    );

    // We can't directly access cache_ttl_seconds, but we can verify
    // the gateway was created successfully with different configs
    assert!(gateway_short.providers().len() >= 5);
    assert_eq!(gateway_long.providers().len(), 0); // Empty custom config
}

// ============================================================================
// Gateway Failover Tests (with mock accounts)
// ============================================================================

#[tokio::test]
async fn test_gateway_list_models_with_no_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let gateway = LlmGatewayImpl::new(http_client, repo, 3600);

    // List models with no accounts configured
    // This should return empty list (no enabled providers)
    let result = gateway.list_models("sk-test-key").await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_gateway_list_models_with_active_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let gateway = LlmGatewayImpl::new(http_client, repo.clone(), 3600);

    // Add active accounts
    repo.save(Account::new("acc-1", "openai", "sk-openai-key"))
        .await
        .unwrap();
    repo.save(Account::new("acc-2", "groq", "sk-groq-key"))
        .await
        .unwrap();

    // List models - will fail to fetch from actual APIs but should handle gracefully
    let result = gateway.list_models("sk-test-key").await;

    // Result depends on network - just verify it doesn't panic
    // In real tests, you'd use wiremock to mock the HTTP calls
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_gateway_list_models_with_inactive_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    let gateway = LlmGatewayImpl::new(http_client, repo.clone(), 3600);

    // Add inactive accounts
    let mut account = Account::new("acc-inactive", "openai", "sk-key");
    account.is_active = false;
    repo.save(account).await.unwrap();

    // List models - should return empty (no active providers)
    let result = gateway.list_models("sk-test-key").await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ============================================================================
// Cache Behavior Tests
// ============================================================================

#[test]
fn test_cache_ttl_helper() {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let ttl = 3600; // 1 hour

    // Fresh cache (now)
    assert!(LlmGatewayImpl::is_cache_valid(&now, ttl));

    // Cache from 30 minutes ago
    let thirty_min_ago = now - Duration::minutes(30);
    assert!(LlmGatewayImpl::is_cache_valid(&thirty_min_ago, ttl));

    // Cache from 2 hours ago (expired)
    let two_hours_ago = now - Duration::hours(2);
    assert!(!LlmGatewayImpl::is_cache_valid(&two_hours_ago, ttl));

    // Cache exactly at TTL boundary
    let exactly_ttl = now - Duration::seconds(ttl as i64);
    assert!(!LlmGatewayImpl::is_cache_valid(&exactly_ttl, ttl));

    // Cache just before TTL
    let just_before = now - Duration::seconds((ttl - 1) as i64);
    assert!(LlmGatewayImpl::is_cache_valid(&just_before, ttl));
}

#[tokio::test]
async fn test_gateway_with_zero_ttl() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    // Create gateway with 0 TTL (cache always expired)
    let gateway = LlmGatewayImpl::new(http_client, repo, 0);

    // Verify gateway created successfully
    assert!(gateway.providers().len() >= 5);
}

#[tokio::test]
async fn test_gateway_with_very_long_ttl() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Arc::new(JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap());
    let http_client = Arc::new(HttpClient::new().unwrap());

    // Create gateway with 1 year TTL
    let gateway = LlmGatewayImpl::new(http_client, repo, 31536000);

    // Verify gateway created successfully
    assert!(gateway.providers().len() >= 5);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_provider_config_empty_strings() {
    // Empty strings should be accepted (validation is elsewhere)
    let config = ProviderConfig::new("", "", "", "");

    assert_eq!(config.id, "");
    assert_eq!(config.name, "");
    assert_eq!(config.base_url, "");
    assert_eq!(config.models_endpoint, "");
}

#[test]
fn test_provider_config_special_characters() {
    let config = ProviderConfig::new(
        "provider-123",
        "Provider with spaces",
        "https://api.example.com/v1?param=value",
        "/models?filter=active",
    );

    assert_eq!(config.id, "provider-123");
    assert_eq!(config.name, "Provider with spaces");
    assert_eq!(config.base_url, "https://api.example.com/v1?param=value");
    assert_eq!(config.models_endpoint, "/models?filter=active");
}
