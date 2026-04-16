//! TUI - Terminal User Interface infrastructure
//!
//! This module provides the infrastructure for communicating state
//! from the async runtime to the TUI thread using tokio's watch channels.

pub mod engine;
pub mod state;

use tokio::sync::watch;

pub use engine::run;
pub use state::{GlobalStats, LogEntry, LogLevel, ProviderMetrics, TuiState};

/// Channel for communicating state to the TUI
///
/// - Sender lives in LlmRouter (or telemetry service)
/// - Receiver is passed to the TUI thread
pub type TuiStateChannel = (watch::Sender<TuiState>, watch::Receiver<TuiState>);

/// Creates a new TUI state channel
pub fn create_tui_channel() -> TuiStateChannel {
    watch::channel(TuiState::default())
}
