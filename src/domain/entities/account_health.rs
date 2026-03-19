//! Account health tracking
//!
//! This module provides health scoring and metrics for accounts.

use crate::app::services::account_rotation::RateLimitInfo;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    Degraded,
}

impl CircuitBreakerState {
    pub fn is_open(&self) -> bool {
        matches!(self, CircuitBreakerState::Open)
    }

    pub fn is_degraded(&self) -> bool {
        matches!(self, CircuitBreakerState::Degraded)
    }
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self::Closed
    }
}

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

    /// Consecutive successes in degraded state (to exit degraded)
    degraded_success_count: u32,

    /// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,

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
            degraded_success_count: 0,
            circuit_breaker_state: CircuitBreakerState::Closed,
            circuit_breaker_opened_at: None,
            quota_remaining: None,
            quota_limit: None,
        }
    }

    /// Returns whether the circuit breaker is open.
    pub fn circuit_breaker_open(&self) -> bool {
        self.circuit_breaker_state.is_open()
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

        // Exit degraded state after 3 consecutive successes
        if self.circuit_breaker_state == CircuitBreakerState::Degraded {
            self.degraded_success_count = self.degraded_success_count.saturating_add(1);
            if self.degraded_success_count >= 3 {
                self.circuit_breaker_state = CircuitBreakerState::Closed;
                self.circuit_breaker_opened_at = None;
                self.degraded_success_count = 0;
            }
        } else if self.circuit_breaker_state != CircuitBreakerState::Closed {
            self.circuit_breaker_state = CircuitBreakerState::Closed;
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
        self.circuit_breaker_state = CircuitBreakerState::Open;
        self.circuit_breaker_opened_at = Some(current_timestamp());
    }

    /// Checks if circuit breaker allows requests.
    /// Auto-closes after 30 seconds.
    /// In DEGRADED state, allows 10% of requests through (canary testing).
    pub fn can_make_request(&mut self) -> bool {
        if !self.circuit_breaker_state.is_open() {
            // In degraded state, allow 10% of requests through
            if self.circuit_breaker_state.is_degraded() {
                return rand_simple() % 10 == 0;
            }
            return true;
        }

        // Check if enough time has passed to try again (30 seconds)
        if let Some(opened_at) = self.circuit_breaker_opened_at {
            let now = current_timestamp();
            if now - opened_at > 30 {
                // Half-open: transition to Degraded for canary testing
                self.circuit_breaker_state = CircuitBreakerState::Degraded;
                self.circuit_breaker_opened_at = None;
                self.degraded_success_count = 0;
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
            if self.circuit_breaker_state == CircuitBreakerState::Closed {
                score += 20.0;
            } else if self.circuit_breaker_state == CircuitBreakerState::Degraded {
                score += 10.0;
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

    /// Updates health from response headers.
    ///
    /// Parses X-RateLimit-* headers and updates quota fields.
    pub fn update_from_headers(&mut self, headers: &[(impl AsRef<str>, impl AsRef<str>)]) {
        for (name, value) in headers {
            let name_lower = name.as_ref().to_lowercase();
            let value_str = value.as_ref();

            match name_lower.as_str() {
                "x-ratelimit-remaining" => {
                    self.quota_remaining = value_str.parse().ok();
                }
                "x-ratelimit-reset" => {
                    // This is typically a Unix timestamp
                }
                "x-ratelimit-limit" => {
                    self.quota_limit = value_str.parse().ok();
                }
                "retry-after" => {
                    // Could be seconds to wait or HTTP date
                    if let Ok(seconds) = value_str.parse::<u64>() {
                        // If it's a number, it's seconds until retry
                    }
                }
                _ => {}
            }
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

fn rand_simple() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_parsing_remaining() {
        let headers = vec![
            ("x-ratelimit-remaining", "95"),
            ("x-ratelimit-limit", "100"),
            ("x-ratelimit-reset", "1234567890"),
        ];

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.remaining, Some(95));
        assert_eq!(info.limit, Some(100));
        assert_eq!(info.reset_timestamp, Some(1234567890));
    }

    #[test]
    fn test_rate_limit_parsing_empty_headers() {
        let headers: Vec<(&str, &str)> = vec![];
        let info = RateLimitInfo::from_headers(&headers);

        assert_eq!(info.remaining, None);
        assert_eq!(info.limit, None);
        assert_eq!(info.reset_timestamp, None);
    }

    #[test]
    fn test_rate_limit_parsing_case_insensitive() {
        let headers = vec![
            ("X-RateLimit-Remaining", "50"),
            ("X-RATELIMIT-LIMIT", "100"),
        ];

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.remaining, Some(50));
        assert_eq!(info.limit, Some(100));
    }

    #[test]
    fn test_rate_limit_parsing_invalid_values() {
        let headers = vec![
            ("x-ratelimit-remaining", "not-a-number"),
            ("x-ratelimit-limit", "also-invalid"),
        ];

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.remaining, None);
        assert_eq!(info.limit, None);
    }

    #[test]
    fn test_circuit_breaker_open_to_degraded() {
        let mut health = AccountHealth::new("test");
        health.circuit_breaker_state = CircuitBreakerState::Closed;
        health.consecutive_failures = 5;
        health.record_failure();

        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Open);

        health.circuit_breaker_opened_at = Some(current_timestamp() - 31);
        let can_request = health.can_make_request();

        assert!(can_request);
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Degraded);
    }

    #[test]
    fn test_circuit_breaker_degraded_to_closed() {
        let mut health = AccountHealth::new("test");
        health.circuit_breaker_state = CircuitBreakerState::Degraded;

        health.record_success(100);
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Degraded);

        health.record_success(100);
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Degraded);

        health.record_success(100);
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_degraded_10_percent_pass_through() {
        let mut health = AccountHealth::new("test");
        health.circuit_breaker_state = CircuitBreakerState::Degraded;

        let mut pass_count = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            let result = health.can_make_request();
            if result {
                pass_count += 1;
            }
        }

        let pass_rate = pass_count as f64 / iterations as f64;
        assert!(
            pass_rate > 0.05 && pass_rate < 0.15,
            "Pass rate {}% should be approximately 10%",
            pass_rate * 100.0
        );
    }

    #[test]
    fn test_circuit_breaker_closed_allows_requests() {
        let mut health = AccountHealth::new("test");
        health.circuit_breaker_state = CircuitBreakerState::Closed;

        assert!(health.can_make_request());
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_open_blocks_requests() {
        let mut health = AccountHealth::new("test");
        health.circuit_breaker_state = CircuitBreakerState::Open;
        health.circuit_breaker_opened_at = Some(current_timestamp());

        assert!(!health.can_make_request());
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Open);
    }

    #[test]
    fn test_account_health_record_success() {
        let mut health = AccountHealth::new("test");
        assert_eq!(health.total_requests, 0);

        health.record_success(100);

        assert_eq!(health.total_requests, 1);
        assert_eq!(health.successful_requests, 1);
        assert_eq!(health.failed_requests, 0);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_account_health_record_failure() {
        let mut health = AccountHealth::new("test");
        assert_eq!(health.consecutive_failures, 0);

        health.record_failure();
        assert_eq!(health.consecutive_failures, 1);

        for _ in 0..4 {
            health.record_failure();
        }
        assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Open);
    }

    #[test]
    fn test_account_health_health_score() {
        let mut health = AccountHealth::new("test");

        let score_new = health.health_score();
        assert!(score_new >= 25.0 && score_new <= 100.0);

        for _ in 0..5 {
            health.record_success(100);
        }
        for _ in 0..3 {
            health.record_failure();
        }

        let score = health.health_score();
        assert!(score >= 0.0 && score <= 100.0);
    }
}
