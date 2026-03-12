//! Chat handler

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::presentation::AppState;

pub fn routes() -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<crate::domain::Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<crate::domain::Choice>,
    pub usage: crate::domain::Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
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
