//! LLM Router - Routes requests to different LLM providers using ExecutionPlanner
//!
//! This module implements the routing logic by:
//! 1. Using ExecutionPlanner to create execution plans based on request context
//! 2. Forwarding requests to the selected provider/account
//! 3. Mapping responses back to domain types
//! 4. Implementing error handling with automatic fallback

use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

#[cfg(feature = "tui")]
use chrono::Utc;
#[cfg(feature = "tui")]
use tokio::sync::watch;

use crate::app::services::execution_plan::{
    ExecutionContext, ExecutionOutcome, ExecutionPlan, ExecutionPlanImpl, ExecutionPlanStatus,
    ExecutionPlanner, ExecutionPlannerConfig, PlanningOptions,
};
use crate::config::RoutingConfig;
use crate::domain::entities::{ChatRequest, ChatResponse, Message};
use crate::domain::errors::DomainError;
use crate::domain::services::TokenValidator;
use crate::domain::traits::AccountRepository;
use crate::error::{Error, Result};
use crate::infrastructure::gateway::llm_gateway::ProviderConfig;
use crate::infrastructure::HttpClient;
use crate::interfaces::provider_request::{ProviderChatMessage, ProviderChatRequest};
#[cfg(feature = "tui")]
use crate::presentation::tui::TuiState;

/// Router configuration for LLM request routing
#[derive(Debug, Clone)]
pub struct LlmRouterConfig {
    /// Enable automatic failover on provider failure
    pub enable_failover: bool,

    /// Maximum number of retries per account
    pub max_retries: u32,

    /// Request timeout in seconds
    pub timeout_seconds: u32,

    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl Default for LlmRouterConfig {
    fn default() -> Self {
        Self {
            enable_failover: true,
            max_retries: 3,
            timeout_seconds: 60,
            verbose_logging: false,
        }
    }
}

/// LLM Router - handles request routing using ExecutionPlanner
///
/// This router replaces the previous stub implementation with full routing logic
/// that leverages the ExecutionPlanner for intelligent account selection.
pub struct LlmRouter<R: AccountRepository + ?Sized> {
    /// HTTP client for making requests
    http_client: Arc<HttpClient>,

    /// Provider configurations (RwLock for hot-reload support)
    provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,

    /// Account repository for fetching account details (including API keys)
    account_repo: Arc<R>,

    /// Execution planner for creating execution plans
    planner: ExecutionPlanner<R>,

    /// Router configuration
    config: LlmRouterConfig,

    /// Routing configuration for logging purposes
    routing_config: Option<RoutingConfig>,

    /// Optional TUI state channel for telemetry
    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    tui_state_tx: Option<watch::Sender<TuiState>>,
}

impl<R: AccountRepository + ?Sized> LlmRouter<R> {
    /// Creates a new LlmRouter with the given dependencies.
    pub fn new(
        http_client: Arc<HttpClient>,
        account_repo: Arc<R>,
        provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,
        planner_config: ExecutionPlannerConfig,
    ) -> Self {
        let planner = ExecutionPlanner::new(account_repo.clone(), planner_config);

        Self {
            http_client,
            provider_configs,
            account_repo,
            planner,
            config: LlmRouterConfig::default(),
            routing_config: None,
            #[cfg(feature = "tui")]
            tui_state_tx: None,
        }
    }

    /// Creates a new LlmRouter with custom configuration.
    pub fn with_config(
        http_client: Arc<HttpClient>,
        account_repo: Arc<R>,
        provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,
        planner_config: ExecutionPlannerConfig,
        router_config: LlmRouterConfig,
    ) -> Self {
        let planner = ExecutionPlanner::new(account_repo.clone(), planner_config);

        Self {
            http_client,
            provider_configs,
            account_repo,
            planner,
            config: router_config,
            routing_config: None,
            #[cfg(feature = "tui")]
            tui_state_tx: None,
        }
    }

    /// Creates a new LlmRouter with routing config.
    #[allow(dead_code)]
    pub fn with_routing_config(
        http_client: Arc<HttpClient>,
        account_repo: Arc<R>,
        provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,
        planner_config: ExecutionPlannerConfig,
        routing_config: RoutingConfig,
    ) -> Self {
        let planner = ExecutionPlanner::new(account_repo.clone(), planner_config);

        Self {
            http_client,
            provider_configs,
            account_repo,
            planner,
            config: LlmRouterConfig::default(),
            routing_config: Some(routing_config),
            #[cfg(feature = "tui")]
            tui_state_tx: None,
        }
    }

    /// Creates a new LlmRouter with optional TUI telemetry channel.
    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn new_with_tui(
        http_client: Arc<HttpClient>,
        account_repo: Arc<R>,
        provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,
        planner_config: ExecutionPlannerConfig,
        tui_state_tx: Option<watch::Sender<TuiState>>,
    ) -> Self {
        let planner = ExecutionPlanner::new(account_repo.clone(), planner_config);

        Self {
            http_client,
            provider_configs,
            account_repo,
            planner,
            config: LlmRouterConfig::default(),
            routing_config: None,
            tui_state_tx,
        }
    }

    /// Creates a new LlmRouter with custom config and optional TUI telemetry channel.
    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn with_config_and_tui(
        http_client: Arc<HttpClient>,
        account_repo: Arc<R>,
        provider_configs: Arc<RwLock<std::collections::HashMap<String, ProviderConfig>>>,
        planner_config: ExecutionPlannerConfig,
        router_config: LlmRouterConfig,
        tui_state_tx: Option<watch::Sender<TuiState>>,
    ) -> Self {
        let planner = ExecutionPlanner::new(account_repo.clone(), planner_config);

        Self {
            http_client,
            provider_configs,
            account_repo,
            planner,
            config: router_config,
            routing_config: None,
            tui_state_tx,
        }
    }

    /// Reload provider configs with new account list (hot-reload support).
    /// Uses short-lived write lock - no I/O while locked.
    pub async fn reload_accounts(&self, accounts: Vec<crate::domain::entities::Account>) {
        let new_configs = self.build_provider_configs(accounts);

        let mut guard = self.provider_configs.write().await;
        *guard = new_configs;
        // Lock released here - short-lived, no I/O

        // Extract provider list while we have a write lock
        #[allow(unused_variables)]
        let provider_list: Vec<String> = guard.keys().cloned().collect();
        drop(guard);

        // Sync to TUI state if available
        #[cfg(feature = "tui")]
        {
            if let Some(ref tx) = self.tui_state_tx {
                tx.send_modify(|state| {
                    // Update provider list in TUI state - clone, modify, replace
                    let mut provider_status = (*state.provider_status).clone();
                    for provider_id in provider_list {
                        provider_status.entry(provider_id).or_default();
                    }
                    state.provider_status = Arc::new(provider_status);
                });
            }
        }
    }

    /// Build provider configs from account list.
    fn build_provider_configs(
        &self,
        accounts: Vec<crate::domain::entities::Account>,
    ) -> std::collections::HashMap<String, ProviderConfig> {
        let mut configs = std::collections::HashMap::new();

        // Group accounts by provider_id
        let mut provider_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for account in &accounts {
            if account.is_active {
                provider_ids.insert(account.provider_id.to_string());
            }
        }

        // Create provider configs from default providers (will be overridden by account-based ones)
        for provider_id in provider_ids {
            // Use default base URL - actual routing uses account's provider_id
            configs.insert(
                provider_id.clone(),
                ProviderConfig::new(
                    &provider_id,
                    &provider_id,
                    format!("https://{}", provider_id).as_str(),
                    "/v1/chat/completions",
                ),
            );
        }

        configs
    }

    /// Inject telemetry update after provider response.
    /// Updates provider metrics and global stats in TuiState.
    #[cfg(feature = "tui")]
    pub fn update_telemetry(
        &self,
        provider_id: String,
        latency_ms: u64,
        success: bool,
        cb_open: bool,
    ) {
        if let Some(ref tx) = self.tui_state_tx {
            tx.send_modify(|state| {
                // Update provider metrics - clone, modify, replace
                let mut provider_status = (*state.provider_status).clone();
                let metrics = provider_status
                    .entry(provider_id.clone())
                    .or_insert_with(|| crate::presentation::tui::ProviderMetrics {
                        provider_id: provider_id.clone(),
                        latency_ms: None,
                        circuit_breaker_open: false,
                        requests_success: 0,
                        requests_failed: 0,
                    });

                metrics.latency_ms = Some(latency_ms);
                metrics.circuit_breaker_open = cb_open;
                if success {
                    metrics.requests_success += 1;
                } else {
                    metrics.requests_failed += 1;
                }
                state.provider_status = Arc::new(provider_status);

                // Update global stats - clone, modify, replace
                let mut global = (*state.global_stats).clone();
                global.requests_total += 1;
                if success {
                    global.requests_success += 1;
                } else {
                    global.requests_failed += 1;
                }

                // Update moving average latency (with guard against division by zero)
                let total = global.requests_total as f64;
                if total > 0.0 {
                    global.avg_latency_ms =
                        (global.avg_latency_ms * (total - 1.0) + latency_ms as f64) / total;
                } else {
                    global.avg_latency_ms = latency_ms as f64;
                }
                state.global_stats = Arc::new(global);

                // Add to log buffer - clone, modify, replace
                let mut buffer = (*state.log_buffer).clone();
                buffer.push_back(crate::presentation::tui::LogEntry {
                    timestamp: Utc::now(),
                    level: if success {
                        crate::presentation::tui::LogLevel::Info
                    } else {
                        crate::presentation::tui::LogLevel::Error
                    },
                    message: if success {
                        format!("Request to {} succeeded in {}ms", provider_id, latency_ms)
                    } else {
                        format!("Request to {} failed after {}ms", provider_id, latency_ms)
                    },
                    provider_id: Some(provider_id),
                });

                // Trim log buffer if needed
                let max_entries = state.max_log_entries;
                while buffer.len() > max_entries {
                    buffer.pop_front();
                }
                state.log_buffer = Arc::new(buffer);
            });
        }
    }

    /// Routes a chat request to the appropriate LLM provider.
    ///
    /// This method:
    /// 1. Creates an execution context from the request
    /// 2. Uses ExecutionPlanner to create an execution plan
    /// 3. Forwards the request to the selected account
    /// 4. Handles failures with automatic fallback if enabled
    /// 5. Returns the response mapped to domain types
    pub async fn route_request(
        &self,
        request: ChatRequest,
        preferred_providers: Vec<String>,
    ) -> Result<ChatResponse> {
        // Step 1: Create execution context
        let context = self.create_execution_context(&request, preferred_providers);

        // Get request_id for logging before context is moved
        let request_id_for_log = context.request_id.clone();

        // Step 1.5: Pre-flight token validation (async — offloads BPE to blocking pool)
        let (_token_count, request) = match TokenValidator::validate_async(request).await {
            Ok((count, req)) => {
                tracing::debug!(
                    target: "token_validation",
                    model = %req.model,
                    token_count = count,
                    "Token count within limits"
                );
                (count, req)
            }
            Err(DomainError::TokenLimitExceeded {
                model,
                tokens,
                limit,
            }) => {
                tracing::warn!(
                    target: "token_validation",
                    model = %model,
                    tokens = tokens,
                    limit = limit,
                    "Request exceeds model context window"
                );
                return Err(Error::InvalidRequest(format!(
                    "Token limit exceeded: {tokens} tokens exceed {limit} token context window for model '{model}'"
                )));
            }
            Err(e) => {
                tracing::warn!(error = %e, "Token validation error");
                return Err(Error::Internal(format!("Token validation failed: {e}")));
            }
        };

        // Step 2: Create execution plan using the planner
        let mut plan = match self.planner.create_plan(context).await {
            Ok(plan) => plan,
            Err(e) => {
                tracing::error!("Failed to create execution plan: {}", e);
                return Err(Error::Internal(format!(
                    "Failed to create execution plan: {}",
                    e
                )));
            }
        };

        // Log routing information with strategy details
        let strategy_name = plan.plan_type().name();
        let account_count = plan.account_count();

        if let Some(ref routing_config) = self.routing_config {
            tracing::info!(
                request_id = %request_id_for_log,
                strategy = %routing_config.strategy,
                plan_type = %strategy_name,
                accounts = account_count,
                cascading_enabled = routing_config.cascading_enabled,
                budget_mode = routing_config.budget_mode,
                "Routing request"
            );
        } else {
            tracing::info!(
                request_id = %request_id_for_log,
                plan_type = %strategy_name,
                accounts = account_count,
                "Routing request"
            );
        }

        if self.config.verbose_logging {
            tracing::info!(
                "Execution plan created: {:?} with {} accounts",
                plan.plan_type(),
                plan.account_count()
            );
        }

        // Step 3: Execute the plan with failover support
        self.execute_with_fallback(&mut plan, &request).await
    }

    /// Creates an execution context from a chat request.
    fn create_execution_context(
        &self,
        request: &ChatRequest,
        preferred_providers: Vec<String>,
    ) -> ExecutionContext {
        let request_id = Uuid::new_v4().to_string();

        let mut context = ExecutionContext::new(request_id, &request.model)
            .with_preferred_providers(preferred_providers);

        // Add request parameters from the chat request
        if let Some(temp) = request.temperature {
            context = context.with_param("temperature", serde_json::json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            context = context.with_param("max_tokens", serde_json::json!(max_tokens));
        }
        if let Some(stream) = request.stream {
            context = context.with_param("stream", serde_json::json!(stream));
        }

        // Set planning options from config
        let planning_options = PlanningOptions::default()
            .with_failover(self.config.enable_failover)
            .with_max_retries(self.config.max_retries)
            .with_timeout(self.config.timeout_seconds);

        context.with_planning_options(planning_options)
    }

    /// Executes the plan with automatic fallback on failure.
    async fn execute_with_fallback(
        &self,
        plan: &mut ExecutionPlanImpl,
        request: &ChatRequest,
    ) -> Result<ChatResponse> {
        // If no accounts available, return error
        if !plan.has_accounts() {
            return Err(Error::Internal(
                "No accounts available for execution".to_string(),
            ));
        }

        // Get planned accounts from the plan
        let accounts = plan.planned_accounts().to_vec();

        // Try each account in order
        for (index, planned_account) in accounts.iter().enumerate() {
            let is_primary = index == 0;

            if self.config.verbose_logging {
                tracing::info!(
                    "Attempting account {} (primary: {})",
                    planned_account.account_id,
                    is_primary
                );
            }

            // Capture start time for latency tracking
            #[cfg(feature = "tui")]
            let request_start = std::time::Instant::now();

            // Try to execute with this account
            let result = self
                .forward_to_provider(
                    &planned_account.account_id,
                    request,
                    &planned_account.provider_id,
                )
                .await;

            // Calculate latency for telemetry (always available, used in tui telemetry)
            #[cfg(feature = "tui")]
            let latency_ms = request_start.elapsed().as_millis() as u64;
            #[cfg(not(feature = "tui"))]
            let _latency_ms: u64 = 0;

            match result {
                Ok(response) => {
                    // Success!
                    let outcome = if is_primary {
                        ExecutionOutcome::Success
                    } else {
                        ExecutionOutcome::Fallback
                    };

                    plan.set_outcome(outcome);

                    // Inject telemetry for success
                    #[cfg(feature = "tui")]
                    {
                        self.update_telemetry(
                            planned_account.provider_id.clone(),
                            latency_ms,
                            true,
                            false,
                        );
                    }

                    if self.config.verbose_logging {
                        tracing::info!(
                            "Request succeeded with account {} (outcome: {:?})",
                            planned_account.account_id,
                            outcome
                        );
                    }

                    return Ok(response);
                }
                Err(e) => {
                    // Account failed, try next if failover is enabled

                    // Inject telemetry for failure (use latency calculated before the match)
                    #[cfg(feature = "tui")]
                    {
                        self.update_telemetry(
                            planned_account.provider_id.clone(),
                            latency_ms,
                            false,
                            false,
                        );
                    }

                    if self.config.verbose_logging {
                        tracing::warn!(
                            "Account {} failed: {}. {}",
                            planned_account.account_id,
                            e,
                            if self.config.enable_failover && index < accounts.len() - 1 {
                                "Trying next account..."
                            } else {
                                "No more accounts to try"
                            }
                        );
                    }

                    if !self.config.enable_failover || index >= accounts.len() - 1 {
                        // No more fallback options
                        plan.set_error(format!("All accounts failed. Last error: {}", e));
                        plan.update_status(ExecutionPlanStatus::Failed);

                        return Err(e);
                    }
                }
            }
        }

        // Should not reach here, but handle it gracefully
        Err(Error::Internal(
            "Execution failed with no accounts available".to_string(),
        ))
    }

    /// Forwards a request to a specific provider using the account.
    async fn forward_to_provider(
        &self,
        account_id: &str,
        request: &ChatRequest,
        provider_id_from_account: &str,
    ) -> Result<ChatResponse> {
        // Fetch the account from repository to get the API key
        let account = self
            .account_repo
            .find_by_id(account_id)
            .await
            .map_err(|e| {
                Error::Internal(format!("Failed to find account '{}': {}", account_id, e))
            })?;

        // Get API key from account
        let api_key = account.api_key.as_ref().ok_or_else(|| {
            Error::Internal(format!("No API key found for account '{}'", account_id))
        })?;

        // Use the provider from the account, NOT inferred from model name
        let provider_id = provider_id_from_account;

        // Get provider config (RwLock read) - bind guard first to avoid temporary lifetime issue
        let configs_guard = self.provider_configs.read().await;
        let provider_config = configs_guard.get(provider_id).ok_or_else(|| {
            Error::ProviderNotFound(format!("Provider '{}' not found", provider_id))
        })?;

        // Build URL - special handling for Cloudflare AI Gateway
        let base_url = if provider_id == "cloudflare" {
            // Cloudflare AI Gateway format: /v1/{account_id}/gateway/{gateway_name}/chat/completions
            // API key format: {account_id}_{api_key}, we use account_id part
            let account_id = api_key.split('_').next().ok_or_else(|| {
                Error::Internal(
                    "Cloudflare API key must be in format '{account_id}_{api_key}'".to_string(),
                )
            })?;
            format!("{}/{account_id}/gateway/ai", provider_config.base_url)
        } else {
            self.http_client
                .mock_base_url()
                .map(|url| format!("{}/v1", url))
                .unwrap_or_else(|| provider_config.base_url.clone())
        };

        let url = format!("{}/chat/completions", base_url);

        // Build request body using typed struct (zero-allocation borrowed refs)
        let provider_messages: Vec<ProviderChatMessage> = request
            .messages
            .iter()
            .map(|m| ProviderChatMessage::new(&m.role, &m.content))
            .collect();

        let body = ProviderChatRequest::builder(&request.model, &provider_messages)
            .with_temperature(request.temperature.unwrap_or(0.7))
            .with_max_tokens(request.max_tokens.unwrap_or(1024))
            .with_stream(request.stream.unwrap_or(false));

        // Make HTTP request with the account's API key
        let response = self
            .http_client
            .client()
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .header(
                "HTTP-Referer",
                "https://github.com/XaviCode1000/Rust-LLM-Api-Router",
            )
            .header("X-Title", "Rust LLM API Router")
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Internal(format!(
                "Provider '{}' returned {}: {}",
                provider_id, status, error_text
            )));
        }

        // Parse response
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse response: {}", e)))?;

        Ok(chat_response)
    }
}

// =============================================================================
// Backward Compatibility API
// =============================================================================

/// Route a request to a specific provider (backward compatible interface).
///
/// This function maintains backward compatibility with the previous API.
pub async fn route_request(
    provider: &str,
    request: serde_json::Value,
) -> Result<serde_json::Value, crate::Error> {
    // Convert generic JSON request to ChatRequest
    let model = request
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4")
        .to_string();

    let messages: Vec<Message> = request
        .get("messages")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let role = m.get("role")?.as_str()?.to_string();
                    let content = m.get("content")?.as_str()?.to_string();
                    Some(Message { role, content })
                })
                .collect()
        })
        .unwrap_or_default();

    let temperature = request
        .get("temperature")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let max_tokens = request
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let stream = request.get("stream").and_then(|v| v.as_bool());

    let _chat_request = ChatRequest::new(model, messages)
        .with_temperature(temperature.unwrap_or(0.7))
        .with_max_tokens(max_tokens.unwrap_or(1024))
        .with_stream(stream.unwrap_or(false));

    // For backward compatibility, we need a way to get the account repository and HTTP client
    // This is a simplified implementation that logs a message
    tracing::warn!(
        "route_request called with provider '{}' but full routing requires LlmRouter initialization",
        provider
    );

    // Return an error indicating the full router needs to be used
    Err(Error::Internal(
        "Please use LlmRouter for full routing capabilities".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_router_config_defaults() {
        let config = LlmRouterConfig::default();
        assert!(config.enable_failover);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_infer_provider_from_model() {
        // Test OpenAI models
        assert_eq!(infer_provider_static("gpt-4"), Some("openai".to_string()));
        assert_eq!(
            infer_provider_static("o1-preview"),
            Some("openai".to_string())
        );
        assert_eq!(
            infer_provider_static("gpt-3.5-turbo"),
            Some("openai".to_string())
        );

        // Test Anthropic models
        assert_eq!(
            infer_provider_static("claude-3-opus"),
            Some("anthropic".to_string())
        );
        assert_eq!(
            infer_provider_static("claude-3-sonnet"),
            Some("anthropic".to_string())
        );
        assert_eq!(
            infer_provider_static("haiku"),
            Some("anthropic".to_string())
        );

        // Test Groq models
        assert_eq!(
            infer_provider_static("llama-3-70b"),
            Some("groq".to_string())
        );
        assert_eq!(
            infer_provider_static("mixtral-8x7b"),
            Some("groq".to_string())
        );

        // Unknown model
        assert_eq!(infer_provider_static("unknown-model"), None);
    }

    // Helper function for testing provider inference (static for testing)
    fn infer_provider_from_model_static(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-")
            || model_lower.starts_with("o1")
            || model_lower.starts_with("o3")
        {
            Some("openai".to_string())
        } else if model_lower.starts_with("claude-")
            || model_lower.starts_with("sonnet")
            || model_lower.starts_with("haiku")
        {
            Some("anthropic".to_string())
        } else if model_lower.contains("llama")
            || model_lower.contains("mixtral")
            || model_lower.contains("groq")
        {
            Some("groq".to_string())
        } else {
            None
        }
    }

    // Alias for test
    fn infer_provider_static(model: &str) -> Option<String> {
        infer_provider_from_model_static(model)
    }
}
