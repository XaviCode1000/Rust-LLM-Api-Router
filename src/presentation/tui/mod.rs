//! TUI - Terminal User Interface infrastructure
//!
//! This module provides the infrastructure for communicating state
//! from the async runtime to the TUI thread using tokio's watch channels.

pub mod engine;
pub mod state;

use tokio::sync::mpsc;
use tokio::sync::watch;

pub use engine::run;
pub use state::{FormState, GlobalStats, InputMode, LogEntry, LogLevel, ProviderMetrics, TuiState};

/// Channel for communicating state to the TUI
///
/// - Sender lives in LlmRouter (or telemetry service)
/// - Receiver is passed to the TUI thread
pub type TuiStateChannel = (watch::Sender<TuiState>, watch::Receiver<TuiState>);

/// Creates a new TUI state channel
pub fn create_tui_channel() -> TuiStateChannel {
    watch::channel(TuiState::default())
}

/// Commands sent from TUI to the core system
#[derive(Debug, Clone)]
pub enum TuiAction {
    /// Add a new account with provider and API key
    AddAccount {
        provider_id: String,
        api_key: String,
    },
    /// Remove an account by ID
    RemoveAccount(String),
    /// Toggle a provider enabled/disabled
    ToggleProvider(String),
    /// Quit the application
    Quit,
}

/// Channel for TuiAction communication (32 buffer to prevent blocking)
pub type TuiActionChannel = (mpsc::Sender<TuiAction>, mpsc::Receiver<TuiAction>);

/// Creates a new TUI action channel
pub fn create_action_channel() -> TuiActionChannel {
    mpsc::channel(32)
}
