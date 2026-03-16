use crate::error::Result;

/// Authentication commands
#[derive(Debug, clap::Subcommand)]
pub enum AuthCommands {
    /// Initiate authentication flow
    Login,

    /// Revoke tokens and clear credentials
    Logout,
}

/// Handle auth subcommand
pub async fn handle_auth_command(cmd: AuthCommands) -> Result<()> {
    match cmd {
        AuthCommands::Login => handle_login_command().await,
        AuthCommands::Logout => handle_logout_command().await,
    }
}

// Import the handler functions from the presentation layer
mod auth_handlers {
    use super::*;

    pub async fn handle_login_command() -> Result<()> {
        crate::presentation::cli::commands::login::handle_login_command().await
    }

    pub async fn handle_logout_command() -> Result<()> {
        crate::presentation::cli::commands::logout::handle_logout_command().await
    }
}

// Re-export the handler functions
pub use auth_handlers::{handle_login_command, handle_logout_command};