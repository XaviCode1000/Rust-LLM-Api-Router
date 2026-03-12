//! Metrics handler

use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::Encoder;

use crate::error::Result;
use crate::presentation::AppState;

pub fn routes() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    
    encoder.encode(&state.metrics.registry.gather(), &mut buffer)
        .map_err(|e| crate::Error::Metrics(e))?;
    
    let output = String::from_utf8(buffer)
        .map_err(|e| crate::Error::Internal(e.to_string()))?;
    
    Ok(([(axum::http::header::CONTENT_TYPE, "text/plain")], output))
}
