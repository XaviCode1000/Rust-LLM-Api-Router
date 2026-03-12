//! Application routes

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::domain::entities::{Choice, Message, Usage};
use crate::presentation::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/metrics", get(metrics))
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, serde::Serialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

#[derive(Debug, serde::Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

async fn chat_completions(
    State(_state): State<AppState>,
    Json(_request): Json<ChatRequest>,
) -> axum::response::Result<Json<ChatResponse>> {
    // Route to appropriate LLM provider
    Err((axum::http::StatusCode::NOT_IMPLEMENTED, "Not implemented").into())
}

async fn list_models(
    State(_state): State<AppState>,
) -> axum::response::Result<Json<ModelsResponse>> {
    Ok(Json(ModelsResponse { data: vec![] }))
}

async fn metrics(State(state): State<AppState>) -> axum::response::Result<String> {
    use prometheus::Encoder;

    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();

    encoder
        .encode(&state.metrics.registry.gather(), &mut buffer)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let output = String::from_utf8(buffer)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(output)
}
