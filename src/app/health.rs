//! Health check endpoint

use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}
