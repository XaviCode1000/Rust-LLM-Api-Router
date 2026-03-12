//! LLM API Router - Main entry point
//!
//! A proxy/router for LLM API requests with support for multiple providers.

use clap::Parser;
use std::net::SocketAddr;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::error::Result;
use rust_llm_api_router::infrastructure::init_logging;
use rust_llm_api_router::presentation::{routes, AppState};

#[derive(Parser, Debug)]
#[command(name = "llm-router")]
#[command(about = "LLM API Router - Proxy for LLM providers")]
struct Args {
    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args.log_level);

    tracing::info!("Starting LLM API Router");

    // Load configuration
    let settings = Settings::default();

    // Override with CLI args
    let mut settings = settings;
    settings.app_host = args.host;
    settings.app_port = args.port;
    settings.log_level = args.log_level;

    // Create application state
    let state = AppState::new(settings)?;

    // Get port before moving state
    let port = state.config.app_port;

    // Build router
    let app = routes();
    let app = app.with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
