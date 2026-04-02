//! Account rotation strategies
//!
//! This module provides different strategies for selecting which account
//! to use when making requests to LLM providers.

use crate::domain::entities::account_health::AccountHealth;
use crate::domain::Account;
use std::collections::HashMap;

/// Account with its associated health data.
///
/// Pairs an account with its health metrics for selection decisions.
pub struct AccountWithHealth<'a> {
    pub account: &'a Account,
    pub health: Option<&'a AccountHealth>,
}

impl<'a> AccountWithHealth<'a> {
    pub fn new(account: &'a Account, health: Option<&'a AccountHealth>) -> Self {
        Self { account, health }
    }

    pub fn avg_latency_ms(&self) -> Option<f64> {
        self.health.map(|h| h.avg_latency_ms)
    }

    pub fn quota_remaining(&self) -> Option<u64> {
        self.health.and_then(|h| h.quota_remaining)
    }

    pub fn is_circuit_breaker_open(&self) -> bool {
        self.health
            .map(|h| h.circuit_breaker_open())
            .unwrap_or(false)
    }
}

/// Configuration for exponential backoff with jitter.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
    pub max_retries: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 100,
            max_delay_ms: 10000,
            jitter_factor: 0.1,
            max_retries: 3,
        }
    }
}

impl BackoffConfig {
    pub fn new(
        base_delay_ms: u64,
        max_delay_ms: u64,
        jitter_factor: f64,
        max_retries: u32,
    ) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms,
            jitter_factor,
            max_retries,
        }
    }

    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let exponential_delay = self.base_delay_ms * (2_u64.pow(attempt));
        let capped_delay = exponential_delay.min(self.max_delay_ms);

        let jitter_multiplier = 1.0 + (rand_simple() as f64 % 2000_f64 / 10000_f64 - 0.1);
        let final_delay = (capped_delay as f64 * jitter_multiplier) as u64;

        final_delay.min(self.max_delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::account_health::{AccountHealth, CircuitBreakerState};

    fn create_test_account(id: &str) -> Account {
        Account::new(id, "test", "test")
            .with_priority(1)
            .with_active(true)
    }

    fn create_test_health(account_id: &str, latency_ms: f64, quota: Option<u64>) -> AccountHealth {
        let mut health = AccountHealth::new(account_id);
        health.avg_latency_ms = latency_ms;
        health.quota_remaining = quota;
        health
    }

    #[test]
    fn test_latency_strategy_selects_lowest_latency() {
        let strategy = LatencyStrategy::new();
        let accounts = vec![
            create_test_account("acc1"),
            create_test_account("acc2"),
            create_test_account("acc3"),
        ];

        let mut health_map = HashMap::new();
        health_map.insert("acc1".to_string(), create_test_health("acc1", 200.0, Some(100)));
        health_map.insert("acc2".to_string(), create_test_health("acc2", 50.0, Some(100)));
        health_map.insert("acc3".to_string(), create_test_health("acc3", 150.0, Some(100)));

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "acc2");
    }

    #[test]
    fn test_latency_strategy_excludes_circuit_breaker_open() {
        let strategy = LatencyStrategy::new();
        let accounts = vec![create_test_account("acc1"), create_test_account("acc2")];

        let mut health_map = HashMap::new();
        health_map.insert("acc1".to_string(), {
            let mut h = create_test_health("acc1", 50.0, Some(100));
            h.circuit_breaker_state = CircuitBreakerState::Open;
            h
        });
        health_map.insert("acc2".to_string(), create_test_health("acc2", 200.0, Some(100)));

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "acc2");
    }

    #[test]
    fn test_latency_strategy_excludes_no_quota_accounts() {
        let strategy = LatencyStrategy::new();
        let accounts = vec![create_test_account("acc1"), create_test_account("acc2")];

        let mut health_map = HashMap::new();
        health_map.insert("acc1".to_string(), create_test_health("acc1", 50.0, Some(0)));
        health_map.insert("acc2".to_string(), create_test_health("acc2", 200.0, Some(100)));

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "acc2");
    }

    #[test]
    fn test_latency_strategy_fallback_without_health() {
        let strategy = LatencyStrategy::new();
        let accounts = vec![create_test_account("acc1"), create_test_account("acc2")];

        let health_map = HashMap::new();

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "acc1");
    }

    #[test]
    fn test_latency_strategy_returns_none_when_all_excluded() {
        let strategy = LatencyStrategy::new();
        let accounts = vec![create_test_account("acc1"), create_test_account("acc2")];

        let mut health_map = HashMap::new();
        health_map.insert("acc1".to_string(), {
            let mut h = create_test_health("acc1", 50.0, Some(0));
            h.circuit_breaker_state = CircuitBreakerState::Open;
            h
        });
        health_map.insert("acc2".to_string(), {
            let mut h = create_test_health("acc2", 100.0, Some(0));
            h.circuit_breaker_state = CircuitBreakerState::Open;
            h
        });

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_none());
    }

    #[test]
    fn test_latency_strategy_empty_accounts() {
        let strategy = LatencyStrategy::new();
        let accounts: Vec<Account> = vec![];
        let health_map = HashMap::new();

        let selected = strategy.select_with_health(&accounts, &health_map);
        assert!(selected.is_none());
    }
}

fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

/// Rate limit information parsed from response headers.
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    pub remaining: Option<u64>,
    pub reset_timestamp: Option<i64>,
    pub limit: Option<u64>,
}

impl RateLimitInfo {
    pub fn from_headers(headers: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        let mut info = RateLimitInfo::default();

        for (name, value) in headers {
            let name_lower = name.as_ref().to_lowercase();
            let value_str = value.as_ref();

            match name_lower.as_str() {
                "x-ratelimit-remaining" => {
                    info.remaining = value_str.parse().ok();
                },
                "x-ratelimit-reset" => {
                    info.reset_timestamp = value_str.parse().ok();
                },
                "x-ratelimit-limit" => {
                    info.limit = value_str.parse().ok();
                },
                _ => {},
            }
        }

        info
    }
}

/// Rotation strategy trait.
///
/// Implementors define how to select the next account from a list of active accounts.
pub trait RotationStrategy: Send + Sync {
    /// Selects the next account from the list.
    ///
    /// # Arguments
    /// * `accounts` - List of active accounts for a provider
    ///
    /// # Returns
    /// The selected account, or None if no accounts available
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account>;

    /// Returns the strategy name.
    fn name(&self) -> &str;

    /// Returns self as Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Round-robin rotation strategy.
///
/// Cycles through accounts sequentially.
pub struct RoundRobinStrategy {
    /// Current index in the rotation
    index: std::sync::atomic::AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationStrategy for RoundRobinStrategy {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        if accounts.is_empty() {
            return None;
        }

        // Atomic increment and get previous value
        let prev = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Wrap around using modulo
        let index = prev % accounts.len();
        accounts.get(index)
    }

    fn name(&self) -> &str {
        "round-robin"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Weighted rotation strategy.
///
/// Selects accounts based on priority (lower priority = more requests).
pub struct WeightedStrategy;

impl WeightedStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WeightedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationStrategy for WeightedStrategy {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        if accounts.is_empty() {
            return None;
        }

        // Accounts are already sorted by priority in find_active_by_provider
        // Select the highest priority (lowest number) account
        accounts.first()
    }

    fn name(&self) -> &str {
        "weighted"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Latency-based rotation strategy.
///
/// Selects the account with the lowest average latency.
/// Requires account health tracking.
pub struct LatencyStrategy;

impl LatencyStrategy {
    pub fn new() -> Self {
        Self
    }

    pub fn select_with_health<'a>(
        &self,
        accounts: &'a [Account],
        health_map: &HashMap<String, AccountHealth>,
    ) -> Option<&'a Account> {
        if accounts.is_empty() {
            return None;
        }

        let mut candidates: Vec<(&'a Account, f64)> = Vec::new();

        for account in accounts {
            if let Some(health) = health_map.get(&account.id) {
                if health.circuit_breaker_open() {
                    continue;
                }
                if let Some(quota) = health.quota_remaining {
                    if quota == 0 {
                        continue;
                    }
                }
                candidates.push((account, health.avg_latency_ms));
            } else {
                candidates.push((account, 0.0));
            }
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates.first().map(|(acc, _)| *acc)
    }
}

impl Default for LatencyStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationStrategy for LatencyStrategy {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        if accounts.is_empty() {
            return None;
        }

        accounts.first()
    }

    fn name(&self) -> &str {
        "latency-based"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// User-affinity rotation strategy.
///
/// Sticks to the same account for the same user/session when possible.
pub struct UserAffinityStrategy {
    /// Last selected account per user/session
    last_selection: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl UserAffinityStrategy {
    pub fn new() -> Self {
        Self {
            last_selection: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Selects an account for a specific user/session.
    pub fn select_for_user<'a>(
        &self,
        accounts: &'a [Account],
        user_id: &str,
    ) -> Option<&'a Account> {
        if accounts.is_empty() {
            return None;
        }

        // Try to get the last used account for this user
        let last = {
            let selection = self.last_selection.lock().ok()?;
            selection.get(user_id).cloned()
        };

        if let Some(last_id) = last {
            // Try to use the same account if still available
            if let Some(account) = accounts.iter().find(|a| a.id == last_id) {
                return Some(account);
            }
        }

        // Fall back to first available account
        accounts.first().map(|account| {
            // Update last selection
            if let Ok(mut selection) = self.last_selection.lock() {
                selection.insert(user_id.to_string(), account.id.clone());
            }
            account
        })
    }
}

impl Default for UserAffinityStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationStrategy for UserAffinityStrategy {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        // For generic selection, use first account
        accounts.first()
    }

    fn name(&self) -> &str {
        "user-affinity"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Account selector that uses a rotation strategy.
pub struct AccountSelector {
    strategy: Box<dyn RotationStrategy>,
    health_map: Option<std::sync::Arc<std::collections::HashMap<String, AccountHealth>>>,
}

impl AccountSelector {
    /// Creates a new selector with the specified strategy.
    pub fn with_strategy(strategy: Box<dyn RotationStrategy>) -> Self {
        Self {
            strategy,
            health_map: None,
        }
    }

    /// Creates a new selector with round-robin strategy.
    pub fn round_robin() -> Self {
        Self {
            strategy: Box::new(RoundRobinStrategy::new()),
            health_map: None,
        }
    }

    /// Creates a new selector with weighted strategy.
    pub fn weighted() -> Self {
        Self {
            strategy: Box::new(WeightedStrategy::new()),
            health_map: None,
        }
    }

    /// Creates a new selector with latency-based strategy.
    pub fn latency_based() -> Self {
        Self {
            strategy: Box::new(LatencyStrategy::new()),
            health_map: None,
        }
    }

    /// Creates a new selector with user-affinity strategy.
    pub fn user_affinity() -> Self {
        Self {
            strategy: Box::new(UserAffinityStrategy::new()),
            health_map: None,
        }
    }

    /// Sets the health map for latency-based selection.
    pub fn with_health_map(
        mut self,
        health_map: std::sync::Arc<std::collections::HashMap<String, AccountHealth>>,
    ) -> Self {
        self.health_map = Some(health_map);
        self
    }

    /// Selects an account from the list using the configured strategy.
    pub fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        if let Some(ref health_map) = self.health_map {
            if let Some(latency_strategy) = self.strategy.as_any().downcast_ref::<LatencyStrategy>()
            {
                return latency_strategy.select_with_health(accounts, health_map);
            }
        }
        self.strategy.select(accounts)
    }

    /// Returns the current strategy name.
    pub fn strategy_name(&self) -> &str {
        self.strategy.name()
    }
}
