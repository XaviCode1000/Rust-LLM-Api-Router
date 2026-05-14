//! Chat completion handler for OpenAI-compatible API
//!
//! This handler processes POST /v1/chat/completions requests.
//! Uses LlmRouter with ExecutionPlanner for intelligent request routing.

use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    Json,
};
use bytes::Bytes;
use futures::Stream;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt as TokioStreamExt;

use crate::domain::traits::LlmGateway;
use crate::domain::{
    Account, ChatRequest as DomainChatRequest, ChatResponse as DomainChatResponse, Message,
    OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice, OpenAIErrorResponse, OpenAIMessage,
    OpenAIUsage,
};
use crate::infrastructure::HttpClient;
use crate::presentation::AppState;
use crate::Result;

/// Handler for POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OpenAIChatRequest>,
) -> Response {
    // Handle streaming vs non-streaming
    if request.stream.unwrap_or(false) {
        return stream_chat_request(State(state), request)
            .await
            .into_response();
    }

    // Process non-streaming request
    match process_chat_request(state, request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Handler for streaming POST /v1/chat/completions requests
async fn stream_chat_request(
    State(state): State<Arc<AppState>>,
    request: OpenAIChatRequest,
) -> Response {
    // Extract model name (format: "provider:model" or just "model")
    let (provider_id, model_name) = parse_model(&request.model);

    // Get active accounts for the provider
    let accounts = match state
        .account_repo
        .find_active_by_provider(&provider_id)
        .await
    {
        Ok(accounts) => accounts,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "provider_error",
                format!(
                    "Failed to get accounts for provider '{}': {}",
                    provider_id, e
                ),
            );
        }
    };

    if accounts.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "no_accounts",
            format!("No active accounts found for provider '{}'", provider_id),
        );
    }

    // Select account using round-robin
    let account = select_account(&accounts, &state);

    // Convert OpenAI request to internal ChatRequest
    let chat_request = convert_to_chat_request(&request, &model_name);

    // Make streaming request to provider using LlmRouter
    // For streaming, we still use the direct provider call since LlmRouter
    // doesn't have streaming support yet - this is a fallback to existing behavior
    let provider_config_guard = state.provider_config.read().await;
    match make_streaming_provider_request(
        &state.http_client,
        &provider_config_guard,
        &account,
        &chat_request,
    )
    .await
    {
        Ok(stream) => {
            // Convert the provider stream to SSE events
            let sse_stream = stream_to_sse_events(stream);

            // Return SSE response with proper headers
            let mut response = Sse::new(sse_stream).into_response();

            // Set SSE-appropriate headers
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/event-stream"),
            );
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            response
                .headers_mut()
                .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));

            response
        }
        Err(e) => error_response(
            StatusCode::BAD_GATEWAY,
            "provider_error",
            format!("Streaming request to provider failed: {}", e),
        ),
    }
}

/// Helper to create error response
fn error_response(status: StatusCode, error_type: &str, message: String) -> Response {
    (status, Json(OpenAIErrorResponse::new(error_type, &message))).into_response()
}

/// Process a chat completion request.
async fn process_chat_request(
    state: Arc<AppState>,
    openai_request: OpenAIChatRequest,
) -> Result<OpenAIChatResponse, OpenAIErrorResponse> {
    // Extract model name (format: "provider:model" or just "model")
    let (provider_id, model_name) = parse_model(&openai_request.model);

    // Convert OpenAI request to internal ChatRequest
    let chat_request = convert_to_chat_request(&openai_request, &model_name);

    // Use LlmRouter with ExecutionPlanner for intelligent routing
    let preferred_providers = if provider_id != "default" {
        vec![provider_id]
    } else {
        vec![]
    };

    match state
        .llm_router
        .route_request(chat_request, preferred_providers)
        .await
    {
        Ok(provider_response) => {
            // Convert provider response to OpenAI format
            Ok(convert_to_openai_response(
                provider_response,
                &openai_request.model,
            ))
        }
        Err(e) => {
            tracing::error!("LlmRouter request failed: {}", e);
            Err(OpenAIErrorResponse::new(
                "provider_error",
                format!("Request to provider failed: {}", e),
            ))
        }
    }
}

/// Parse model string to extract provider ID and model name.
/// Supports formats: "provider:model", "provider/model", or just "model"
pub fn parse_model(model: &str) -> (String, String) {
    // Try colon separator first (OpenRouter style)
    if let Some(pos) = model.find(':') {
        let provider = &model[..pos];
        let model_name = &model[pos + 1..];
        return (provider.to_string(), model_name.to_string());
    }

    // Try slash separator
    if let Some(pos) = model.find('/') {
        let provider = &model[..pos];
        let model_name = &model[pos + 1..];
        return (provider.to_string(), model_name.to_string());
    }

    // Default: use model as-is, will try to match with any provider
    ("default".to_string(), model.to_string())
}

/// Select an account from the list using simple rotation.
fn select_account(accounts: &[Account], _state: &AppState) -> Account {
    // Simple round-robin using atomic counter
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % accounts.len();
    accounts[index].clone()
}

/// Convert OpenAI request to internal ChatRequest format.
fn convert_to_chat_request(
    openai_request: &OpenAIChatRequest,
    model_name: &str,
) -> DomainChatRequest {
    let messages: Vec<Message> = openai_request
        .messages
        .iter()
        .map(|m| Message {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    // Use the full model string (provider:model) as the model for the provider
    // The provider will understand its own model names
    DomainChatRequest::new(model_name, messages)
        .with_temperature(openai_request.temperature.unwrap_or(0.7))
        .with_max_tokens(openai_request.max_tokens.unwrap_or(1024))
        .with_stream(openai_request.stream.unwrap_or(false))
}

/// Get the base URL for a provider from ProviderConfig.
/// Falls back to mock URL if configured for testing, then to hardcoded URLs.
fn get_provider_base_url(
    http_client: &HttpClient,
    provider_config: &std::collections::HashMap<
        String,
        crate::infrastructure::gateway::llm_gateway::ProviderConfig,
    >,
    provider_id: &str,
) -> String {
    // Check if mock URL is configured (for testing) - highest priority
    if let Some(mock_url) = http_client.mock_base_url() {
        return format!("{}/v1", mock_url);
    }

    // Try to get URL from ProviderConfig
    if let Some(config) = provider_config.get(provider_id) {
        return config.base_url.clone();
    }

    // Fallback to hardcoded URLs for backward compatibility
    match provider_id {
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
        "mistral" => "https://api.mistral.ai/v1".to_string(),
        "cerebras" => "https://api.cerebras.ai/v1".to_string(),
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        _ => provider_id.to_string(), // Use as-is if not recognized
    }
}

/// Make HTTP request to the provider.
#[allow(dead_code)]
async fn make_provider_request(
    http_client: &HttpClient,
    provider_config: &std::collections::HashMap<
        String,
        crate::infrastructure::gateway::llm_gateway::ProviderConfig,
    >,
    account: &Account,
    chat_request: &DomainChatRequest,
) -> Result<DomainChatResponse, String> {
    // Build provider URL using ProviderConfig
    let base_url =
        get_provider_base_url(http_client, provider_config, account.provider_id.as_str());
    let url = format!("{}/chat/completions", base_url);

    // Build request body in OpenAI format (what providers expect)
    let body = serde_json::json!({
        "model": chat_request.model,
        "messages": chat_request.messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect::<Vec<_>>(),
        "temperature": chat_request.temperature,
        "max_tokens": chat_request.max_tokens,
        "stream": chat_request.stream.unwrap_or(false)
    });

    // Make HTTP POST request
    let access_token = account.get_access_token().ok_or_else(|| {
        format!(
            "No authentication credentials available for account '{}'",
            account.id
        )
    })?;

    let response = http_client
        .client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    // Check for error status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Provider returned {}: {}", status, error_text));
    }

    // Parse response
    let chat_response: DomainChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(chat_response)
}

/// Make HTTP streaming request to the provider.
async fn make_streaming_provider_request(
    http_client: &HttpClient,
    provider_config: &std::collections::HashMap<
        String,
        crate::infrastructure::gateway::llm_gateway::ProviderConfig,
    >,
    account: &Account,
    chat_request: &DomainChatRequest,
) -> Result<impl Stream<Item = Result<Bytes, String>>, String> {
    // Build provider URL using ProviderConfig
    let base_url =
        get_provider_base_url(http_client, provider_config, account.provider_id.as_str());
    let url = format!("{}/chat/completions", base_url);

    // Build request body in OpenAI format (what providers expect)
    let body = serde_json::json!({
        "model": chat_request.model,
        "messages": chat_request.messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect::<Vec<_>>(),
        "temperature": chat_request.temperature,
        "max_tokens": chat_request.max_tokens,
        "stream": true
    });

    // Make HTTP POST request with streaming
    let access_token = account.get_access_token().ok_or_else(|| {
        format!(
            "No authentication credentials available for account '{}'",
            account.id
        )
    })?;

    let response = http_client
        .client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    // Check for error status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Provider returned {}: {}", status, error_text));
    }

    // Get the streaming body and convert error type from reqwest::Error to String
    let bytes_stream = response
        .bytes_stream()
        .map_err(|e| format!("Stream error: {}", e));

    Ok(bytes_stream)
}

/// Convert provider byte stream to SSE events.
/// This is a passthrough implementation - we pass through the raw SSE data
/// from the provider directly to the client with minimal parsing.
fn stream_to_sse_events(
    stream: impl Stream<Item = Result<Bytes, String>>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    // Transform each chunk into an SSE event with the raw data
    stream.map(|result| {
        match result {
            Ok(bytes) => {
                // Convert bytes to string, passing through raw SSE data
                match String::from_utf8(bytes.to_vec()) {
                    Ok(data) => {
                        // Skip empty chunks
                        if data.trim().is_empty() {
                            Ok(Event::default().data(""))
                        } else {
                            Ok(Event::default().data(data))
                        }
                    }
                    Err(_) => {
                        // Binary data or invalid UTF-8 - convert to hex representation
                        let hex = bytes
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        Ok(Event::default().data(format!("[binary: {}]", hex)))
                    }
                }
            }
            Err(e) => {
                // On error, send an error message as SSE data
                Ok(Event::default().data(format!("[error: {}]", e)))
            }
        }
    })
}

/// Convert internal ChatResponse to OpenAI format.
pub fn convert_to_openai_response(
    chat_response: DomainChatResponse,
    model: &str,
) -> OpenAIChatResponse {
    let choices: Vec<OpenAIChoice> = chat_response
        .choices
        .into_iter()
        .map(|choice| {
            OpenAIChoice::new(
                choice.index,
                OpenAIMessage {
                    role: choice.message.role,
                    content: choice.message.content,
                    name: None,
                },
                choice.finish_reason.as_deref(),
            )
        })
        .collect();

    let usage = OpenAIUsage::new(
        chat_response.usage.prompt_tokens,
        chat_response.usage.completion_tokens,
        chat_response.usage.total_tokens,
    );

    OpenAIChatResponse::new(
        format!("chatcmpl-{}", chat_response.id),
        model,
        choices,
        usage,
    )
}

/// Handler for GET /v1/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OpenAIModelsResponse>, (StatusCode, Json<OpenAIErrorResponse>)> {
    // Use the first available API key from active accounts
    let api_key = match get_api_key_for_models(&state).await {
        Some(key) => key,
        None => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(OpenAIErrorResponse::new(
                    "no_api_key",
                    "No API keys configured for any provider".to_string(),
                )),
            ));
        }
    };

    match state.llm_gateway.list_models(&api_key).await {
        Ok(models) => {
            let now = chrono::Utc::now().timestamp() as u64;
            let data: Vec<OpenAIModelInfo> = models
                .into_iter()
                .map(|m| OpenAIModelInfo {
                    id: format!("{}:{}", m.provider_id, m.id),
                    object: "model".to_string(),
                    created: now,
                    owned_by: m.provider_id,
                })
                .collect();

            Ok(Json(OpenAIModelsResponse {
                object: "list".to_string(),
                data,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch models: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OpenAIErrorResponse::new(
                    "models_error",
                    format!("Failed to fetch models: {}", e),
                )),
            ))
        }
    }
}

/// Helper to get an API key from the first active account
pub async fn get_api_key_for_models(state: &AppState) -> Option<String> {
    match state.account_repo.find_active().await {
        Ok(accounts) => accounts
            .into_iter()
            .next()
            .and_then(|a| a.auth_method.api_key().map(|s| s.to_string())),
        Err(e) => {
            tracing::warn!("Failed to fetch accounts for models endpoint: {}", e);
            None
        }
    }
}

/// Response for /v1/models endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelsResponse {
    pub object: String,
    pub data: Vec<OpenAIModelInfo>,
}

/// Model information for /v1/models response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::gateway::llm_gateway::ProviderConfig;
    use std::collections::HashMap;

    #[test]
    fn test_get_provider_base_url_uses_mock_url() {
        let http_client = HttpClient::with_mock_url("http://localhost:8080").unwrap();
        let provider_config = HashMap::new();

        let url = get_provider_base_url(&http_client, &provider_config, "openai");

        // Mock URL should take precedence
        assert_eq!(url, "http://localhost:8080/v1");
    }

    #[test]
    fn test_get_provider_base_url_uses_provider_config() {
        let http_client = HttpClient::new().unwrap();
        let mut provider_config = HashMap::new();
        provider_config.insert(
            "custom".to_string(),
            ProviderConfig::new("custom", "Custom", "https://custom.api.com/v1", "/models"),
        );

        let url = get_provider_base_url(&http_client, &provider_config, "custom");

        assert_eq!(url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_get_provider_base_url_fallback_hardcoded() {
        let http_client = HttpClient::new().unwrap();
        let provider_config = HashMap::new();

        let url = get_provider_base_url(&http_client, &provider_config, "openai");

        // Should fallback to hardcoded URL
        assert_eq!(url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_get_provider_base_url_unknown_provider() {
        let http_client = HttpClient::new().unwrap();
        let provider_config = HashMap::new();

        let url = get_provider_base_url(&http_client, &provider_config, "unknown-provider");

        // Unknown provider returns the provider ID as-is
        assert_eq!(url, "unknown-provider");
    }
}
