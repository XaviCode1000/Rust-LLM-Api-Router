//! Spinner wrapper for long-running CLI operations.
//!
//! Shows an animated spinner while async operations (validation, login, etc.)
//! are in progress. Automatically disables when not running in a TTY.

use crate::presentation::cli::tty::is_tty;
use indicatif::{ProgressBar, ProgressStyle};

/// Animated spinner for CLI operations.
///
/// Automatically clears itself on drop.
pub struct CliSpinner {
    pb: Option<ProgressBar>,
}

impl CliSpinner {
    /// Create a new spinner with the given message.
    ///
    /// If not running in a TTY, returns a no-op spinner.
    #[must_use]
    pub fn new(message: &str) -> Self {
        if !is_tty() {
            return Self { pb: None };
        }
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Self { pb: Some(pb) }
    }

    /// Finish the spinner and display a final message.
    pub fn finish_with_message(&self, message: &str) {
        if let Some(ref pb) = self.pb {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Abandon the spinner without a final message.
    pub fn abandon(&self) {
        if let Some(ref pb) = self.pb {
            pb.abandon();
        }
    }
}

impl Drop for CliSpinner {
    fn drop(&mut self) {
        if let Some(ref pb) = self.pb {
            pb.finish_and_clear();
        }
    }
}
