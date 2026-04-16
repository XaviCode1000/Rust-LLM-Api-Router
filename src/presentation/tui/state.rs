//! TUI State - State structures for the Terminal User Interface
//!
//! This module provides lightweight state structures designed for efficient
//! communication between the async runtime and the TUI thread via watch channels.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Metrics per provider
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub provider_id: String,
    pub latency_ms: Option<u64>,
    pub circuit_breaker_open: bool,
    pub requests_success: u64,
    pub requests_failed: u64,
}

/// Global statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GlobalStats {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub avg_latency_ms: f64,
    pub cost_micro_dollars: u64,
}

/// Log entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub provider_id: Option<String>,
}

/// Log level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// TUI State - Lightweight state for watch::channel
///
/// This struct is designed to be lightweight for efficient
/// communication via tokio's watch channel.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TuiState {
    pub provider_status: HashMap<String, ProviderMetrics>,
    pub global_stats: GlobalStats,
    pub log_buffer: VecDeque<LogEntry>,
    #[serde(skip)]
    max_log_entries: usize,
}

impl TuiState {
    /// Creates a new TuiState with specified log buffer capacity
    /// If max_log_entries is 0, defaults to 100
    pub fn new(max_log_entries: usize) -> Self {
        let max = if max_log_entries == 0 {
            100
        } else {
            max_log_entries
        };
        Self {
            provider_status: HashMap::new(),
            global_stats: GlobalStats::default(),
            log_buffer: VecDeque::with_capacity(max),
            max_log_entries: max,
        }
    }

    /// Add a log entry with bounded buffer
    pub fn add_log(&mut self, entry: LogEntry) {
        if self.log_buffer.len() >= self.max_log_entries {
            self.log_buffer.pop_front();
        }
        self.log_buffer.push_back(entry);
    }

    /// Update provider metrics (fire-and-forget style)
    pub fn update_provider(&mut self, provider_id: String, metrics: ProviderMetrics) {
        self.provider_status.insert(provider_id, metrics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_state_new() {
        let state = TuiState::new(50);
        assert!(state.provider_status.is_empty());
        assert!(state.global_stats.requests_total == 0);
        assert!(state.log_buffer.is_empty());
    }

    #[test]
    fn test_add_log_bounded() {
        let mut state = TuiState::new(3);

        // Add more logs than capacity
        for i in 0..5 {
            state.add_log(LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                message: format!("Log {}", i),
                provider_id: None,
            });
        }

        // Should have at most 3 entries
        assert!(state.log_buffer.len() <= 3);
    }

    #[test]
    fn test_update_provider() {
        let mut state = TuiState::default();

        let metrics = ProviderMetrics {
            provider_id: "openai".to_string(),
            latency_ms: Some(150),
            circuit_breaker_open: false,
            requests_success: 10,
            requests_failed: 2,
        };

        state.update_provider("openai".to_string(), metrics.clone());

        let stored = state.provider_status.get("openai");
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().requests_success, 10);
    }
}
