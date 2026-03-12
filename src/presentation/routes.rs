//! Application routes

use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::interfaces::handlers::{chat_completions, list_models};
use crate::presentation::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/metrics", get(metrics))
}

async fn health() -> &'static str {
    "OK"
}

async fn metrics(State(state): State<Arc<AppState>>) -> axum::response::Result<String> {
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
