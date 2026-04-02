//! Shared CLI input helpers

use crate::presentation::cli::prompt::prompt_secret;
use crate::Result;

/// Prompt the user to enter an API key via secure interactive input.
///
/// Uses masked input when running in a TTY, otherwise returns an error.
pub fn read_api_key_interactive() -> Result<String> {
    prompt_secret("Enter API Key:")
}
