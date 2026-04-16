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
use rust_llm_api_router::config::RoutingConfig;
use rust_llm_api_router::presentation::cli::Cli;
use std::net::SocketAddr;
use std::panic;
use std::sync::Arc;

use rust_llm_api_router::config::Settings;
use rust_llm_api_router::error::Result;
use rust_llm_api_router::infrastructure::init_logging;
use rust_llm_api_router::presentation::{routes, AppState};

fn main() {
    // P0: Terminal restoration on panic
    panic::set_hook(Box::new(|info| {
        // Restore terminal - use crossterm to leave raw mode and alternate screen
        #[cfg(feature = "tui")]
        {
            use crossterm::{terminal::LeaveAlternateScreen, ExecutableCommand};
            let _ = std::io::stdout().execute(LeaveAlternateScreen);
            // Raw mode is automatically restored on drop, but force it
            let _ = crossterm::terminal::disable_raw_mode();
        }
        eprintln!("PANIC: {}", info);
    }));

    // Server mode entry point
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    if let Err(e) = runtime.block_on(async { run_server().await }) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run_server() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level);

    // Handle CLI subcommands
    if let Some(commands) = cli.commands {
        return rust_llm_api_router::presentation::cli::handle_command(commands).await;
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

    // Create routing config from CLI args and environment variables
    let routing_config = match RoutingConfig::from_cli_and_env(
        &cli.routing_strategy,
        cli.cascading,
        cli.quality_threshold,
        cli.budget_mode,
        cli.max_retries,
        cli.timeout,
    ) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    };

    tracing::info!(
        "Routing config: strategy={}, cascading={}, budget_mode={}, max_retries={}, timeout={}s",
        routing_config.strategy,
        routing_config.cascading_enabled,
        routing_config.budget_mode,
        routing_config.max_retries,
        routing_config.timeout_seconds
    );

    // Create application state with routing config
    let state = AppState::new(settings, routing_config)?;
    let state = Arc::new(state);

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
