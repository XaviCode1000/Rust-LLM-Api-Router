//! TUI State - State structures for the Terminal User Interface
//!
//! This module provides lightweight state structures designed for efficient
//! communication between the async runtime and the TUI thread via watch channels.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Input mode for the TUI - controls keyboard handling behavior
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum InputMode {
    /// Normal navigation mode - arrow keys, tabs, etc.
    #[default]
    Normal,
    /// Editing mode - typing in form fields
    Editing,
    /// Processing mode - waiting for async validation (shows spinner)
    Processing,
}

/// Form data being edited (cleared after submission for security)
#[derive(Clone, Debug, Default)]
pub struct FormState {
    /// Provider ID being configured
    pub provider_id: String,
    /// API key buffer - only exists in memory, never persisted
    pub api_key_buffer: String,
    /// Cursor position in the form field
    pub cursor_position: usize,
    /// Validation error message if any
    pub validation_error: Option<String>,
}

impl FormState {
    /// Clear sensitive data - called after submission or cancel
    pub fn clear(&mut self) {
        self.api_key_buffer.clear();
        self.provider_id.clear();
        self.cursor_position = 0;
        self.validation_error = None;
    }
}

/// Metrics per provider
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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
/// Uses Arc-wrapped fields for cheap clones in watch channel.
///
/// Note: Serialize/Deserialize implemented manually to handle Arc wrapping.
#[derive(Clone, Debug, Default)]
pub struct TuiState {
    /// BTreeMap for ordered iteration (eliminates visual jumping)
    pub provider_status: Arc<BTreeMap<String, ProviderMetrics>>,
    pub global_stats: Arc<GlobalStats>,
    pub log_buffer: Arc<VecDeque<LogEntry>>,
    /// Latency history for sparkline (50 items)
    pub latency_history: Arc<VecDeque<u64>>,
    pub max_log_entries: usize,
    /// Current input mode for keyboard handling
    pub input_mode: InputMode,
    /// Form data being edited (sensitive - cleared after submission)
    pub form_state: FormState,
    /// Processing timeout - reverts to Normal after expiry
    pub processing_timeout: Option<std::time::Instant>,
}

// Custom serialization to unwrap Arc for JSON
// Note: form_state and processing_timeout are NOT serialized (security - sensitive data)
impl Serialize for TuiState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TuiState", 6)?;
        state.serialize_field("provider_status", &*self.provider_status)?;
        state.serialize_field("global_stats", &*self.global_stats)?;
        state.serialize_field("log_buffer", &*self.log_buffer)?;
        state.serialize_field("latency_history", &*self.latency_history)?;
        state.serialize_field("max_log_entries", &self.max_log_entries)?;
        state.serialize_field("input_mode", &self.input_mode)?;
        state.end()
    }
}

// Custom deserialization to re-wrap in Arc
// Note: form_state and processing_timeout initialized to defaults (not serialized)
impl<'de> Deserialize<'de> for TuiState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TuiStateRaw {
            provider_status: BTreeMap<String, ProviderMetrics>,
            global_stats: GlobalStats,
            log_buffer: VecDeque<LogEntry>,
            latency_history: VecDeque<u64>,
            max_log_entries: usize,
            input_mode: Option<InputMode>,
        }

        let raw = TuiStateRaw::deserialize(deserializer)?;
        Ok(Self {
            provider_status: Arc::new(raw.provider_status),
            global_stats: Arc::new(raw.global_stats),
            log_buffer: Arc::new(raw.log_buffer),
            latency_history: Arc::new(raw.latency_history),
            max_log_entries: raw.max_log_entries,
            input_mode: raw.input_mode.unwrap_or_default(),
            form_state: FormState::default(),
            processing_timeout: None,
        })
    }
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
            provider_status: Arc::new(BTreeMap::new()),
            global_stats: Arc::new(GlobalStats::default()),
            log_buffer: Arc::new(VecDeque::with_capacity(max)),
            latency_history: Arc::new(VecDeque::with_capacity(50)),
            max_log_entries: max,
            input_mode: InputMode::default(),
            form_state: FormState::default(),
            processing_timeout: None,
        }
    }

    /// Add a log entry with bounded buffer - returns new Arc-wrapped state
    #[must_use]
    pub fn add_log(&self, entry: LogEntry) -> Self {
        let mut new_buffer = (*self.log_buffer).clone();
        if new_buffer.len() >= self.max_log_entries {
            new_buffer.pop_front();
        }
        new_buffer.push_back(entry);

        Self {
            provider_status: self.provider_status.clone(),
            global_stats: self.global_stats.clone(),
            log_buffer: Arc::new(new_buffer),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: self.form_state.clone(),
            processing_timeout: self.processing_timeout,
        }
    }

    /// Update provider metrics - returns new Arc-wrapped state
    #[must_use]
    pub fn update_provider(&self, provider_id: String, metrics: ProviderMetrics) -> Self {
        let mut new_map = (*self.provider_status).clone();
        new_map.insert(provider_id, metrics);

        Self {
            provider_status: Arc::new(new_map),
            global_stats: self.global_stats.clone(),
            log_buffer: self.log_buffer.clone(),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: self.form_state.clone(),
            processing_timeout: self.processing_timeout,
        }
    }

    /// Update global stats - returns new Arc-wrapped state
    #[must_use]
    pub fn update_stats(&self, stats: GlobalStats) -> Self {
        Self {
            provider_status: self.provider_status.clone(),
            global_stats: Arc::new(stats),
            log_buffer: self.log_buffer.clone(),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: self.form_state.clone(),
            processing_timeout: self.processing_timeout,
        }
    }

    /// Add latency sample - returns new Arc-wrapped state
    #[must_use]
    pub fn add_latency(&self, latency_ms: u64) -> Self {
        let mut new_history = (*self.latency_history).clone();
        if new_history.len() >= 50 {
            new_history.pop_front();
        }
        new_history.push_back(latency_ms);

        Self {
            provider_status: self.provider_status.clone(),
            global_stats: self.global_stats.clone(),
            log_buffer: self.log_buffer.clone(),
            latency_history: Arc::new(new_history),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: self.form_state.clone(),
            processing_timeout: self.processing_timeout,
        }
    }

    /// Set input mode - returns new Arc-wrapped state
    #[must_use]
    pub fn set_input_mode(&self, mode: InputMode) -> Self {
        Self {
            provider_status: self.provider_status.clone(),
            global_stats: self.global_stats.clone(),
            log_buffer: self.log_buffer.clone(),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: mode,
            form_state: self.form_state.clone(),
            processing_timeout: self.processing_timeout,
        }
    }

    /// Set form state - returns new Arc-wrapped state
    #[must_use]
    pub fn set_form_state(&self, form: FormState) -> Self {
        Self {
            provider_status: self.provider_status.clone(),
            global_stats: self.global_stats.clone(),
            log_buffer: self.log_buffer.clone(),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: form,
            processing_timeout: self.processing_timeout,
        }
    }

    /// Set processing timeout - returns new Arc-wrapped state
    /// Used when transitioning to Processing mode
    #[must_use]
    pub fn with_processing_timeout(&self, timeout: std::time::Instant) -> Self {
        Self {
            provider_status: self.provider_status.clone(),
            global_stats: self.global_stats.clone(),
            log_buffer: self.log_buffer.clone(),
            latency_history: self.latency_history.clone(),
            max_log_entries: self.max_log_entries,
            input_mode: self.input_mode.clone(),
            form_state: self.form_state.clone(),
            processing_timeout: Some(timeout),
        }
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
        let state = TuiState::new(3);

        // Add more logs than capacity
        let new_state = (0..5).fold(state, |acc, i| {
            acc.add_log(LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                message: format!("Log {}", i),
                provider_id: None,
            })
        });

        // Should have at most 3 entries
        assert!(new_state.log_buffer.len() <= 3);
    }

    #[test]
    fn test_update_provider() {
        let state = TuiState::default();

        let metrics = ProviderMetrics {
            provider_id: "openai".to_string(),
            latency_ms: Some(150),
            circuit_breaker_open: false,
            requests_success: 10,
            requests_failed: 2,
        };

        let new_state = state.update_provider("openai".to_string(), metrics.clone());

        let stored = new_state.provider_status.get("openai");
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().requests_success, 10);
    }

    #[test]
    fn test_add_latency_bounded() {
        let state = TuiState::new(50);

        // Add more latencies than capacity
        let new_state = (0..60).fold(state, |acc, i| acc.add_latency(i as u64));

        // Should have at most 50 entries
        assert!(new_state.latency_history.len() <= 50);
    }

    #[test]
    fn test_btree_map_sorted_order() {
        let state = TuiState::new(50);

        // Insert in unsorted order
        let state = state.update_provider("zebra".to_string(), ProviderMetrics::default());
        let state = state.update_provider("apple".to_string(), ProviderMetrics::default());
        let state = state.update_provider("mango".to_string(), ProviderMetrics::default());

        // Get keys - should be sorted
        let keys: Vec<_> = state.provider_status.keys().collect();
        assert_eq!(keys, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let state = TuiState::new(50)
            .update_provider("test".to_string(), ProviderMetrics::default())
            .update_stats(GlobalStats {
                requests_total: 100,
                requests_success: 95,
                requests_failed: 5,
                avg_latency_ms: 150.0,
                cost_micro_dollars: 1000,
            })
            .add_latency(100);

        // Roundtrip through JSON
        let json = serde_json::to_string(&state).unwrap();
        let restored: TuiState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.provider_status.get("test"), Some(&ProviderMetrics::default()));
        assert_eq!(restored.global_stats.requests_total, 100);
        assert_eq!(restored.latency_history.len(), 1);
    }

    #[test]
    fn test_input_mode_default() {
        let mode = InputMode::default();
        assert_eq!(mode, InputMode::Normal);
    }

    #[test]
    fn test_input_mode_transitions() {
        assert!(InputMode::Normal == InputMode::Normal);
        assert!(InputMode::Editing != InputMode::Normal);
    }

    #[test]
    fn test_form_state_clear() {
        let mut form = FormState {
            provider_id: "test-provider".to_string(),
            api_key_buffer: "secret-key-123".to_string(),
            cursor_position: 10,
            validation_error: Some("Invalid key".to_string()),
        };

        form.clear();

        assert!(form.provider_id.is_empty());
        assert!(form.api_key_buffer.is_empty());
        assert_eq!(form.cursor_position, 0);
        assert!(form.validation_error.is_none());
    }

    #[test]
    fn test_set_input_mode() {
        let state = TuiState::new(50);
        assert_eq!(state.input_mode, InputMode::Normal);

        let editing_state = state.set_input_mode(InputMode::Editing);
        assert_eq!(editing_state.input_mode, InputMode::Editing);

        let processing_state = editing_state.set_input_mode(InputMode::Processing);
        assert_eq!(processing_state.input_mode, InputMode::Processing);
    }

    #[test]
    fn test_set_form_state() {
        let state = TuiState::new(50);
        let mut form = FormState::default();
        form.provider_id = "openai".to_string();
        form.api_key_buffer = "sk-test".to_string();

        let new_state = state.set_form_state(form.clone());

        assert_eq!(new_state.form_state.provider_id, "openai");
        assert_eq!(new_state.form_state.api_key_buffer, "sk-test");
    }

    #[test]
    fn test_with_processing_timeout() {
        let state = TuiState::new(50);
        let timeout = std::time::Instant::now();
        let new_state = state.with_processing_timeout(timeout);

        assert!(new_state.processing_timeout.is_some());
    }
}
