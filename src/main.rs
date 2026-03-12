//! LLM API Router - Main entry point
//!
//! A proxy/router for LLM API requests with support for multiple providers.
//!
//! # Usage
//!
//! ## Server Mode
//!
//! ```bash
//! llm-router --port 8080
//! ```
//!
//! ## CLI Commands
//!
//! ```bash
//! llm-router provider add --id openai --name "OpenAI" --base-url https://api.openai.com/v1
//! llm-router provider list
//! llm-router provider validate --id openai
//! ```

use clap::Parser;
use rust_llm_api_router::cli::Cli;
use std::net::SocketAddr;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::error::Result;
use rust_llm_api_router::infrastructure::init_logging;
use rust_llm_api_router::presentation::{routes, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level);

    // Handle CLI subcommands
    if let Some(commands) = cli.commands {
        return rust_llm_api_router::cli::handle_command(commands).await;
    }

    // Server mode
    tracing::info!("Starting LLM API Router");

    // Load configuration
    let settings = Settings::default();

    // Override with CLI args
    let mut settings = settings;
    settings.app_host = cli.host;
    settings.app_port = cli.port;
    settings.log_level = cli.log_level;

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
