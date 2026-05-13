//! Interactive prompt utilities for CLI.
//!
//! Provides TTY-aware wrappers around inquire for secure and confirm prompts.
//! Falls back to defaults or returns errors when not running in a terminal.

use crate::presentation::cli::tty::is_tty;
use crate::Result;
use inquire::{Confirm, Password};

/// Prompt the user for a yes/no confirmation.
///
/// If not running in a TTY, returns the default value (false) silently.
pub fn confirm(message: &str) -> Result<bool> {
    if !is_tty() {
        // Non-interactive mode: return default
        return Ok(false);
    }

    Confirm::new(message)
        .with_default(false)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}

/// Prompt the user to enter a secret (masked input, e.g., API key).
///
/// Returns an error if not running in a TTY, as interactive input is required.
pub fn prompt_secret(message: &str) -> Result<String> {
    if !is_tty() {
        return Err(crate::Error::Internal(
            "Interactive input requires a terminal".to_string(),
        ));
    }

    Password::new(message)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}

/// Prompt the user to enter text (visible input).
///
/// Returns an error if not running in a TTY.
pub fn prompt_text(message: &str) -> Result<String> {
    if !is_tty() {
        return Err(crate::Error::Internal(
            "Interactive input requires a terminal".to_string(),
        ));
    }

    inquire::Text::new(message)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}
