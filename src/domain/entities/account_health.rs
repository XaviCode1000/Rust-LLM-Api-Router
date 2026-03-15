//! Account health tracking
//!
//! This module provides health scoring and metrics for accounts.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Health metrics for an account.
///
/// Tracks success rate, latency, and other metrics for account rotation decisions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountHealth {
    /// Account ID this health belongs to
    pub account_id: String,

    /// Total requests made
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Last 100 response times in milliseconds
    #[serde(skip)]
    pub recent_latencies: VecDeque<u64>,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Last successful request timestamp
    pub last_success_at: Option<u64>,

    /// Last failed request timestamp
    pub last_failure_at: Option<u64>,

    /// Consecutive failures count (for circuit breaker)
    pub consecutive_failures: u32,

    /// Circuit breaker state (open = not allowing requests)
    pub circuit_breaker_open: bool,

    /// When circuit breaker was opened (timestamp in seconds)
    pub circuit_breaker_opened_at: Option<u64>,

    /// Quota remaining (if provider supports it, None = unknown)
    pub quota_remaining: Option<u64>,

    /// Quota limit (if provider supports it, None = unknown)
    pub quota_limit: Option<u64>,
}

impl AccountHealth {
    /// Creates a new AccountHealth with default values.
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            recent_latencies: VecDeque::with_capacity(100),
            avg_latency_ms: 0.0,
            last_success_at: None,
            last_failure_at: None,
            consecutive_failures: 0,
            circuit_breaker_open: false,
            circuit_breaker_opened_at: None,
            quota_remaining: None,
            quota_limit: None,
        }
    }

    /// Records a successful request.
    pub fn record_success(&mut self, latency_ms: u64) {
        // Use saturating_add to prevent overflow (CWE-190)
        self.total_requests = self.total_requests.saturating_add(1);
        self.successful_requests = self.successful_requests.saturating_add(1);
        self.consecutive_failures = 0;

        // Update latency tracking
        self.recent_latencies.push_back(latency_ms);
        if self.recent_latencies.len() > 100 {
            self.recent_latencies.pop_front();
        }
        self.update_avg_latency();

        // Update timestamps
        self.last_success_at = Some(current_timestamp());

        // Close circuit breaker if it was open
        if self.circuit_breaker_open {
            self.circuit_breaker_open = false;
            self.circuit_breaker_opened_at = None;
        }
    }

    /// Records a failed request.
    pub fn record_failure(&mut self) {
        // Use saturating_add to prevent overflow (CWE-190)
        self.total_requests = self.total_requests.saturating_add(1);
        self.failed_requests = self.failed_requests.saturating_add(1);
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.last_failure_at = Some(current_timestamp());

        // Open circuit breaker after 5 consecutive failures
        if self.consecutive_failures >= 5 {
            self.open_circuit_breaker();
        }
    }

    /// Opens the circuit breaker.
    fn open_circuit_breaker(&mut self) {
        self.circuit_breaker_open = true;
        self.circuit_breaker_opened_at = Some(current_timestamp());
    }

    /// Checks if circuit breaker allows requests.
    /// Auto-closes after 30 seconds.
    pub fn can_make_request(&mut self) -> bool {
        if !self.circuit_breaker_open {
            return true;
        }

        // Check if enough time has passed to try again (30 seconds)
        if let Some(opened_at) = self.circuit_breaker_opened_at {
            let now = current_timestamp();
            if now - opened_at > 30 {
                // Half-open: allow one request to test
                self.circuit_breaker_open = false;
                self.circuit_breaker_opened_at = None;
                return true;
            }
        }

        false
    }

    /// Updates the average latency from recent latencies.
    fn update_avg_latency(&mut self) {
        if self.recent_latencies.is_empty() {
            self.avg_latency_ms = 0.0;
        } else {
            let sum: u64 = self.recent_latencies.iter().sum();
            self.avg_latency_ms = sum as f64 / self.recent_latencies.len() as f64;
        }
    }

    /// Calculates health score (0-100).
    ///
    /// Higher is better. Based on:
    /// - Success rate (50 points max)
    /// - Latency (30 points max) - only scored if there are recorded latencies
    /// - Circuit breaker state (20 points max) - only scored if there are requests
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;

        // Success rate (50 points)
        if self.total_requests > 0 {
            let success_rate = self.successful_requests as f64 / self.total_requests as f64;
            score += success_rate * 50.0;

            // Latency score (30 points) - only scored if we have latency data
            // < 500ms = 30 points, 500-2000ms = 15 points, > 2000ms = 0 points
            if !self.recent_latencies.is_empty() {
                if self.avg_latency_ms < 500.0 {
                    score += 30.0;
                } else if self.avg_latency_ms < 2000.0 {
                    score += 15.0;
                }
            }

            // Circuit breaker (20 points) - only scored after requests
            if !self.circuit_breaker_open {
                score += 20.0;
            }
        } else {
            score += 25.0; // Default score for new accounts (no requests yet)
        }

        score.min(100.0)
    }

    /// Returns success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            100.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Returns quota usage as a percentage.
    pub fn quota_usage(&self) -> Option<f64> {
        match (self.quota_remaining, self.quota_limit) {
            (Some(remaining), Some(limit)) if limit > 0 => {
                Some(((limit - remaining) as f64 / limit as f64) * 100.0)
            }
            _ => None,
        }
    }
}

/// Returns current timestamp in seconds since epoch.
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
