pub mod commands;
pub mod input;
pub mod output;
pub mod prompt;
pub mod spinner;
pub mod table;
pub mod tty;

use clap::Parser;

pub use commands::account::AccountCommands;
pub use commands::auth::AuthCommands;
pub use commands::provider::ProviderCommands;

use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};

/// CLI command enum
#[derive(Debug, Parser)]
#[command(name = "llm-router")]
#[command(about = "LLM API Router - Proxy for LLM providers")]
#[command(after_help = r#"
ROUTING STRATEGIES:
    auto            Planner decides based on context (default)
    cost-optimized  Always select cheapest capable model
    cascading       Start cheap, escalate if quality is low
    failover        Sequential fallback on failure
    load-balanced   Health-weighted distribution

EXAMPLES:
    llm-router --routing-strategy cascading --quality-threshold 0.85
    llm-router --budget-mode --max-retries 5
    llm-router --routing-strategy failover --timeout 30
"#)]
pub struct Cli {
    /// Host to bind to (server mode)
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Port to bind to (server mode)
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Routing strategy: auto, cost-optimized, cascading, failover, load-balanced
    #[arg(long, default_value = "auto")]
    pub routing_strategy: String,

    /// Enable cascading (quality-based escalation)
    #[arg(long)]
    pub cascading: bool,

    /// Minimum quality score for cascading (0.0-1.0)
    #[arg(long, default_value = "0.75")]
    pub quality_threshold: f64,

    /// Enable budget mode (select cheapest model)
    #[arg(long)]
    pub budget_mode: bool,

    /// Maximum retries per request
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Request timeout in seconds
    #[arg(long, default_value = "60")]
    pub timeout: u64,

    /// CLI subcommands
    #[command(subcommand)]
    pub commands: Option<CliCommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum CliCommands {
    /// Provider management commands
    #[command(subcommand)]
    Provider(ProviderCommands),

    /// Account management commands
    #[command(subcommand)]
    Account(AccountCommands),

    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Generate shell completions
    #[cfg(feature = "completions")]
    #[command(subcommand)]
    Completions(commands::completions::CompletionsCommands),
}

/// Handle CLI commands
pub async fn handle_command(command: CliCommands) -> crate::error::Result<()> {
    // Initialize TTY detection for coloured output
    tty::init();

    match command {
        CliCommands::Provider(provider_cmd) => {
            let repo =
                JsonProviderRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            let account_repo =
                JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            commands::provider::handle_provider_command(provider_cmd, &repo, &account_repo).await
        },
        CliCommands::Account(account_cmd) => {
            let repo =
                JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            commands::account::handle_account_command(account_cmd, &repo).await
        },
        CliCommands::Auth(auth_cmd) => commands::auth::handle_auth_command(auth_cmd).await,
        #[cfg(feature = "completions")]
        CliCommands::Completions(cmd) => commands::completions::handle_completions_command(cmd),
    }
}
