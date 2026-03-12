use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, sse::Sse},
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio::stream::Stream;
use futures::stream::{self, StreamExt};

use crate::application::services::llm_service::{LlmService, LlmServiceImpl};
use crate::domain::entities::{
    ChatRequest, ChatResponse, Model, Provider,
};
use crate::infrastructure::errors::AppError;

// Handler for chat completions endpoint
pub async fn chat_handler(
    State(service): State<LlmServiceImpl>,
    AxumJson(payload): AxumJson<ChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if streaming is requested (default to false if not set)
    let stream = payload.stream.unwrap_or(false);
    
    if stream {
        // Create a channel for streaming
        let (tx, rx) = mpsc::channel(100);
        
        // Spawn a task to handle the streaming
        tokio::spawn(async move {
            if let Err(e) = service.stream_chat_completion(payload, tx).await {
                // Send error as an SSE event
                let _ = tx.send(Err(format!("Stream error: {}", e))).await;
            }
        });
        
        // Convert the receiver stream to an SSE stream
        let sse_stream = rx.map(|result| {
            result.map_err(|e| axum::Error::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e,
            )))
        });
        
        Ok(Sse::new(sse_stream))
    } else {
        // Non-streaming response
        let response = service.chat_completion(payload).await?;
        Ok((StatusCode::OK, AxumJson(response)))
    }
}

// Handler for listing models
pub async fn list_models_handler(
    State(service): State<LlmServiceImpl>,
) -> Result<impl IntoResponse, AppError> {
    let models = service.list_models().await?;
    Ok((StatusCode::OK, AxumJson(models)))
}

// Handler for listing providers
pub async fn providers_list_handler(
    State(service): State<LlmServiceImpl>,
) -> Result<impl IntoResponse, AppError> {
    let providers = service.list_providers().await?;
    Ok((StatusCode::OK, AxumJson(providers)))
}

// Handler for creating a provider
pub async fn providers_create_handler(
    State(service): State<LlmServiceImpl>,
    AxumJson(payload): AxumJson<Provider>,
) -> Result<impl IntoResponse, AppError> {
    let provider = service.create_provider(payload).await?;
    Ok((StatusCode::CREATED, AxumJson(provider)))
}

// Handler for health check
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

// Handler for metrics endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    // In a real implementation, this would return Prometheus metrics
    (StatusCode::OK, "# HELP llm_proxy_requests_total Total number of requests\n# TYPE llm_proxy_requests_total counter\nllm_proxy_requests_total 0".to_string())
}

// Import json! macro for health handler
#[macro_use]
extern crate serde_json;
use serde_json::json;