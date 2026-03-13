//! Chat completion handler for OpenAI-compatible API
//!
//! This handler processes POST /v1/chat/completions requests.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::traits::AccountRepository;
use crate::domain::{
    Account, ChatRequest, ChatResponse, Message, OpenAIChatRequest, OpenAIChatResponse,
    OpenAIChoice, OpenAIErrorResponse, OpenAIMessage, OpenAIUsage,
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
        // TODO: Implement streaming
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(OpenAIErrorResponse::new(
                "not_implemented",
                "Streaming not yet implemented",
            )),
        )
            .into_response();
    }

    // Process non-streaming request
    match process_chat_request(state, request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Process a chat completion request.
async fn process_chat_request(
    state: Arc<AppState>,
    openai_request: OpenAIChatRequest,
) -> Result<OpenAIChatResponse, OpenAIErrorResponse> {
    // Extract model name (format: "provider:model" or just "model")
    let (provider_id, model_name) = parse_model(&openai_request.model);

    // Get active accounts for the provider
    let accounts = state
        .account_repo
        .find_active_by_provider(&provider_id)
        .await
        .map_err(|e| {
            OpenAIErrorResponse::new(
                "provider_error",
                format!(
                    "Failed to get accounts for provider '{}': {}",
                    provider_id, e
                ),
            )
        })?;

    if accounts.is_empty() {
        return Err(OpenAIErrorResponse::new(
            "no_accounts",
            format!("No active accounts found for provider '{}'", provider_id),
        ));
    }

    // Select account using round-robin (from FailoverManager or simple selection)
    let account = select_account(&accounts, &state);

    // Convert OpenAI request to internal ChatRequest
    let chat_request = convert_to_chat_request(&openai_request, &model_name);

    // Make request to provider
    match make_provider_request(&state.http_client, &account, &chat_request).await {
        Ok(provider_response) => {
            // Convert provider response to OpenAI format
            Ok(convert_to_openai_response(
                provider_response,
                &openai_request.model,
            ))
        }
        Err(e) => Err(OpenAIErrorResponse::new(
            "provider_error",
            format!("Request to provider failed: {}", e),
        )),
    }
}

/// Parse model string to extract provider ID and model name.
/// Supports formats: "provider:model", "provider/model", or just "model"
fn parse_model(model: &str) -> (String, String) {
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
fn convert_to_chat_request(openai_request: &OpenAIChatRequest, model_name: &str) -> ChatRequest {
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
    ChatRequest::new(model_name, messages)
        .with_temperature(openai_request.temperature.unwrap_or(0.7))
        .with_max_tokens(openai_request.max_tokens.unwrap_or(1024))
        .with_stream(openai_request.stream.unwrap_or(false))
}

/// Make HTTP request to the provider.
async fn make_provider_request(
    http_client: &HttpClient,
    account: &Account,
    chat_request: &ChatRequest,
) -> Result<ChatResponse, String> {
    // Build provider URL - use base_url from provider info
    let base_url = match account.provider_id.as_str() {
        "groq" => "https://api.groq.com/openai/v1",
        "openrouter" => "https://openrouter.ai/api/v1",
        "mistral" => "https://api.mistral.ai/v1",
        "cerebras" => "https://api.cerebras.ai/v1",
        "openai" => "https://api.openai.com/v1",
        _ => &account.provider_id, // Use as-is if not recognized
    };

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
    let response = http_client
        .client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", account.api_key))
        .header("Content-Type", "application/json")
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
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(chat_response)
}

/// Convert internal ChatResponse to OpenAI format.
fn convert_to_openai_response(chat_response: ChatResponse, model: &str) -> OpenAIChatResponse {
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
async fn get_api_key_for_models(state: &AppState) -> Option<String> {
    match state.account_repo.find_active().await {
        Ok(accounts) => accounts.into_iter().next().map(|a| a.api_key),
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
