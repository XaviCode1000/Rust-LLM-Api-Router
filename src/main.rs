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

#[cfg(feature = "tui")]
use rust_llm_api_router::presentation::tui::{
    create_action_channel, create_tui_channel, TuiAction,
};
#[cfg(feature = "tui")]
use std::thread;
#[cfg(feature = "tui")]
use tokio::sync::mpsc;

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

/// Process TuiAction commands from the TUI
#[cfg(feature = "tui")]
async fn tui_action_processor(
    mut rx: mpsc::Receiver<TuiAction>,
    account_repo: Arc<dyn rust_llm_api_router::domain::traits::AccountRepository>,
    router: Arc<
        rust_llm_api_router::app::router::llm_router::LlmRouter<
            dyn rust_llm_api_router::domain::traits::AccountRepository + 'static,
        >,
    >,
    state_tx: tokio::sync::watch::Sender<rust_llm_api_router::presentation::tui::TuiState>,
) {
    while let Some(action) = rx.recv().await {
        match action {
            TuiAction::AddAccount {
                provider_id,
                api_key,
            } => {
                // Persist to repository
                let account = rust_llm_api_router::domain::entities::Account::new(
                    provider_id.clone(),
                    provider_id.clone(),
                    api_key,
                );
                if let Err(e) = account_repo.save(account).await {
                    // Log error to TUI
                    state_tx.send_modify(|s| {
                        let mut buffer = (*s.log_buffer).clone();
                        buffer.push_back(rust_llm_api_router::presentation::tui::LogEntry {
                            timestamp: chrono::Utc::now(),
                            level: rust_llm_api_router::presentation::tui::LogLevel::Error,
                            message: format!("Failed to add account {}: {}", provider_id, e),
                            provider_id: None,
                        });
                        s.log_buffer = Arc::new(buffer);
                    });
                    continue;
                }

                // Reload router
                let accounts = account_repo.find_all().await.unwrap_or_default();
                router.reload_accounts(accounts).await;

                // Send confirmation log
                state_tx.send_modify(|s| {
                    let mut buffer = (*s.log_buffer).clone();
                    buffer.push_back(rust_llm_api_router::presentation::tui::LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: rust_llm_api_router::presentation::tui::LogLevel::Info,
                        message: format!("Account {} added successfully", provider_id),
                        provider_id: None,
                    });
                    s.log_buffer = Arc::new(buffer);
                });
            }
            TuiAction::RemoveAccount(account_id) => {
                // Delete from repository
                if let Err(e) = account_repo.delete(&account_id).await {
                    // Log error to TUI
                    state_tx.send_modify(|s| {
                        let mut buffer = (*s.log_buffer).clone();
                        buffer.push_back(rust_llm_api_router::presentation::tui::LogEntry {
                            timestamp: chrono::Utc::now(),
                            level: rust_llm_api_router::presentation::tui::LogLevel::Error,
                            message: format!("Failed to remove account {}: {}", account_id, e),
                            provider_id: None,
                        });
                        s.log_buffer = Arc::new(buffer);
                    });
                    continue;
                }

                // Reload router
                let accounts = account_repo.find_all().await.unwrap_or_default();
                router.reload_accounts(accounts).await;

                // Send confirmation log
                state_tx.send_modify(|s| {
                    let mut buffer = (*s.log_buffer).clone();
                    buffer.push_back(rust_llm_api_router::presentation::tui::LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: rust_llm_api_router::presentation::tui::LogLevel::Info,
                        message: format!("Account {} removed successfully", account_id),
                        provider_id: None,
                    });
                    s.log_buffer = Arc::new(buffer);
                });
            }
            TuiAction::ToggleProvider(provider_id) => {
                // Toggle provider enabled state
                // For now, just log the action
                state_tx.send_modify(|s| {
                    let mut buffer = (*s.log_buffer).clone();
                    buffer.push_back(rust_llm_api_router::presentation::tui::LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: rust_llm_api_router::presentation::tui::LogLevel::Info,
                        message: format!("Provider {} toggled", provider_id),
                        provider_id: Some(provider_id),
                    });
                    s.log_buffer = Arc::new(buffer);
                });
            }
            TuiAction::Quit => {
                // Quit signal - break the loop
                break;
            }
        }
    }
}

/// Handle SIGINT/SIGTERM for graceful shutdown
#[cfg(feature = "tui")]
async fn signal_handler(action_tx: mpsc::Sender<TuiAction>) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Error setting up signal handler: {}", e);
        return;
    }

    // Send quit signal to TUI
    let _ = action_tx.send(TuiAction::Quit).await;
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
        }
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
    let state = AppState::new(settings, routing_config.clone())?;
    let state = Arc::new(state);

    // Get port before moving state
    let port = state.config.app_port;

    // Build router
    let app = routes();
    let app = app.with_state(state.clone());

    #[cfg(feature = "tui")]
    {
        // Create TUI channels
        let (tui_state_tx, tui_state_rx) = create_tui_channel();
        let (action_tx, action_rx) = create_action_channel();

        // Create TUI-enabled router with telemetry channel
        let tui_router = Arc::new(
            rust_llm_api_router::app::router::llm_router::LlmRouter::with_config_and_tui(
                state.http_client.clone(),
                state.account_repo.clone(),
                state.provider_config.clone(),
                rust_llm_api_router::app::services::execution_plan::ExecutionPlannerConfig::from_routing_config(&routing_config),
                rust_llm_api_router::app::router::llm_router::LlmRouterConfig::default(),
                Some(tui_state_tx.clone()),
            )
        );

        // Spawn TuiAction processor
        let processor_handle = tokio::spawn(tui_action_processor(
            action_rx,
            state.account_repo.clone(),
            tui_router.clone(),
            tui_state_tx.clone(),
        ));

        // Spawn signal handler
        let signal_handle = tokio::spawn(signal_handler(action_tx.clone()));

        // Clone action_tx for TUI thread
        let tui_action_tx = action_tx.clone();
        // Spawn TUI thread
        let tui_handle = thread::spawn(move || {
            let result = rust_llm_api_router::presentation::tui::run(tui_state_rx, tui_action_tx);
            // Convert error to string to make it Send
            result.map_err(|e| e.to_string())
        });

        // Start server
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

        tracing::info!("Server listening on {}", addr);
        tracing::info!("TUI enabled - press Ctrl+C to exit");

        // Run server with graceful shutdown
        let server_future = axum::serve(listener, app);

        // Wait for server to complete (will complete on error or when listener drops)
        let _ = server_future.await;

        // Signal TUI to quit
        let _ = action_tx.send(TuiAction::Quit).await;

        // Wait for TUI thread to clean up
        let _ = tui_handle.join();

        // Clean up processor and signal handler
        processor_handle.abort();
        signal_handle.abort();
    }

    #[cfg(not(feature = "tui"))]
    {
        // Start server without TUI
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

        tracing::info!("Server listening on {}", addr);

        axum::serve(listener, app).await?;
    }

    Ok(())
}
