//! Shared CLI input helpers

use crate::Result;
use std::io::{self, BufRead, Write};

/// Prompt the user to enter an API key via stdin.
///
/// Prints a prompt, flushes stdout, and reads a single line from stdin.
/// Returns the trimmed input as a `String`.
pub fn read_api_key_interactive() -> Result<String> {
    print!("Enter API Key: ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    Ok(line.trim().to_string())
}
