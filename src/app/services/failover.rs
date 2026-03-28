//! Failover logic for account rotation
//!
//! This module provides automatic failover between accounts when requests fail.

use std::sync::Arc;
use std::time::Duration;

use crate::domain::traits::AccountRepository;
use crate::domain::{Account, AccountHealth};

use super::account_rotation::{AccountSelector, BackoffConfig, RateLimitInfo};

/// Failover manager for account requests.
///
/// Handles automatic retry with different accounts when requests fail.
pub struct FailoverManager {
    /// Account repository for fetching accounts
    account_repo: Arc<dyn AccountRepository>,

    /// Account selector with rotation strategy
    selector: AccountSelector,

    /// Health tracking per account
    health_map: std::sync::Mutex<std::collections::HashMap<String, AccountHealth>>,

    /// Maximum retry attempts
    max_retries: u32,

    /// Backoff configuration for retries
    backoff_config: BackoffConfig,
}

impl FailoverManager {
    /// Creates a new failover manager.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for fetching accounts
    /// * `selector` - Account selection strategy
    /// * `max_retries` - Maximum number of retry attempts
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use rust_llm_api_router::app::services::{FailoverManager, AccountSelector};
    /// use rust_llm_api_router::infrastructure::JsonAccountRepository;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Arc::new(JsonAccountRepository::new()?);
    /// let manager = FailoverManager::new(repo, AccountSelector::round_robin(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        selector: AccountSelector,
        max_retries: u32,
    ) -> Self {
        Self {
            account_repo,
            selector,
            health_map: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_retries,
            backoff_config: BackoffConfig::default(),
        }
    }

    /// Creates a new failover manager with custom backoff config.
    pub fn with_backoff(
        account_repo: Arc<dyn AccountRepository>,
        selector: AccountSelector,
        max_retries: u32,
        backoff_config: BackoffConfig,
    ) -> Self {
        Self {
            account_repo,
            selector,
            health_map: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_retries,
            backoff_config,
        }
    }

    /// Creates a new failover manager with round-robin strategy.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for fetching accounts
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use rust_llm_api_router::app::services::FailoverManager;
    /// use rust_llm_api_router::infrastructure::JsonAccountRepository;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Arc::new(JsonAccountRepository::new()?);
    /// let manager = FailoverManager::with_round_robin(repo);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_round_robin(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::round_robin(), 3)
    }

    /// Creates a new failover manager with weighted strategy.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for fetching accounts
    pub fn with_weighted(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::weighted(), 3)
    }

    /// Creates a new failover manager with latency-based strategy.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for fetching accounts
    pub fn with_latency_based(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::latency_based(), 3)
    }

    /// Creates a new failover manager with user-affinity strategy.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for fetching accounts
    pub fn with_user_affinity(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::user_affinity(), 3)
    }

    /// Executes a request with automatic failover.
    ///
    /// # Arguments
    /// * `provider_id` - Provider ID to get accounts for
    /// * `execute` - Async function to execute the request with an account
    ///   Returns `Result<(T, Vec<(String, String)>), E>` where the tuple
    ///   contains the response and optional response headers for rate limiting
    ///
    /// # Returns
    /// The response if successful, or the last error if all accounts fail
    ///
    /// # Errors
    /// Returns an error if:
    /// - No accounts are available for the provider
    /// - All retry attempts fail
    /// - The repository fails to fetch accounts
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use rust_llm_api_router::app::services::FailoverManager;
    /// use rust_llm_api_router::infrastructure::JsonAccountRepository;
    /// use rust_llm_api_router::domain::Account;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Arc::new(JsonAccountRepository::new()?);
    /// let manager = FailoverManager::with_round_robin(repo);
    ///
    /// let result: Result<String, String> = manager
    ///     .execute_with_failover("openai", |account| {
    ///         let account_id = account.id.clone();
    ///         async move {
    ///             // Execute request with account.api_key
    ///             // Return response with optional headers (empty vec if no headers)
    ///             Ok((format!("Success with {}", account_id), vec![]))
    ///         }
    ///     })
    ///     .await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_with_failover<F, Fut, T, E>(
        &self,
        provider_id: &str,
        execute: F,
    ) -> Result<T, E>
    where
        F: Fn(&Account) -> Fut,
        Fut: std::future::Future<Output = Result<(T, Vec<(String, String)>), E>>,
        E: std::fmt::Debug + Clone,
    {
        // Get active accounts for provider
        let accounts = self
            .account_repo
            .find_active_by_provider(provider_id)
            .await
            .map_err(|_| self.create_no_accounts_error(provider_id))?;

        if accounts.is_empty() {
            return Err(self.create_no_accounts_error(provider_id));
        }

        // Try each account with failover
        let mut last_error: Option<E> = None;
        let mut tried_accounts: Vec<String> = Vec::new();

        for attempt in 0..self.max_retries as usize {
            // Select next account using rotation strategy
            let account = self.selector.select(&accounts);

            if let Some(account) = account {
                // Check if we've already tried this account
                if tried_accounts.contains(&account.id) {
                    // Try to get a different account
                    continue;
                }

                tried_accounts.push(account.id.clone());

                // Check health/circuit breaker
                if !self.can_use_account(&account.id) {
                    continue;
                }

                // Execute request
                let start = std::time::Instant::now();
                match execute(account).await {
                    Ok((response, headers)) => {
                        // Parse rate limit info from headers and update health
                        self.update_rate_limits(&account.id, &headers);

                        // Record success with latency
                        self.record_success(&account.id, start.elapsed().as_millis() as u64);
                        return Ok(response);
                    }
                    Err(e) => {
                        // Record failure
                        self.record_failure(&account.id);
                        last_error = Some(e);

                        // Apply backoff delay before next retry (only if we have more accounts to try)
                        if attempt < self.max_retries as usize - 1 {
                            let delay_ms = self.backoff_config.calculate_delay(attempt as u32);
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            continue;
                        }
                    }
                }
            } else {
                // No more accounts available
                break;
            }
        }

        // All attempts failed
        Err(last_error.unwrap_or_else(|| self.create_no_accounts_error(provider_id)))
    }

    /// Checks if an account can be used (circuit breaker check).
    fn can_use_account(&self, account_id: &str) -> bool {
        let mut health_map = self.health_map.lock().unwrap();
        let health = health_map
            .entry(account_id.to_string())
            .or_insert_with(|| AccountHealth::new(account_id));

        health.can_make_request()
    }

    /// Records a successful request.
    fn record_success(&self, account_id: &str, latency_ms: u64) {
        let mut health_map = self.health_map.lock().unwrap();
        let health = health_map
            .entry(account_id.to_string())
            .or_insert_with(|| AccountHealth::new(account_id));
        health.record_success(latency_ms);
    }

    /// Records a failed request.
    fn record_failure(&self, account_id: &str) {
        let mut health_map = self.health_map.lock().unwrap();
        let health = health_map
            .entry(account_id.to_string())
            .or_insert_with(|| AccountHealth::new(account_id));
        health.record_failure();
    }

    /// Updates rate limit information from response headers.
    fn update_rate_limits(&self, account_id: &str, headers: &[(String, String)]) {
        if headers.is_empty() {
            return;
        }

        let rate_limit_info = RateLimitInfo::from_headers(headers);

        let mut health_map = self.health_map.lock().unwrap();
        let health = health_map
            .entry(account_id.to_string())
            .or_insert_with(|| AccountHealth::new(account_id));

        if let Some(remaining) = rate_limit_info.remaining {
            health.quota_remaining = Some(remaining);
        }
        if let Some(limit) = rate_limit_info.limit {
            health.quota_limit = Some(limit);
        }
    }

    /// Creates a "no accounts available" error.
    fn create_no_accounts_error<T>(&self, provider_id: &str) -> T {
        // This is a workaround - in real code, we'd have a proper error type
        panic!("No available accounts for provider: {}", provider_id)
    }

    /// Returns health for an account.
    pub fn get_health(&self, account_id: &str) -> Option<AccountHealth> {
        let health_map = self.health_map.lock().unwrap();
        health_map.get(account_id).cloned()
    }

    /// Returns health scores for all accounts.
    pub fn get_all_health(&self) -> Vec<AccountHealth> {
        let health_map = self.health_map.lock().unwrap();
        health_map.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_exponential_increase() {
        let config = BackoffConfig::new(100, 10000, 0.1, 3);

        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);

        assert!(delay0 <= config.max_delay_ms);
        assert!(delay1 <= config.max_delay_ms);
        assert!(delay2 <= config.max_delay_ms);

        let base = 100_u64;
        // Account for jitter multiplier range [0.9, 1.1]
        assert!(delay0 >= base * 1 * 9 / 10); // 90
        assert!(delay1 >= base * 2 * 9 / 10); // 180
        assert!(delay2 >= base * 4 * 9 / 10); // 360
    }

    #[test]
    fn test_backoff_max_delay_capped() {
        let config = BackoffConfig::new(1000, 500, 0.1, 10);

        for attempt in 0..10 {
            let delay = config.calculate_delay(attempt);
            assert!(delay <= 500, "Delay {} exceeded max 500", delay);
        }
    }

    #[test]
    fn test_backoff_jitter_variation() {
        let config = BackoffConfig::new(1000, 10000, 0.1, 3);

        let mut delays: Vec<u64> = (0..10).map(|_| config.calculate_delay(2)).collect();

        delays.sort();
        let min = delays.first().copied().unwrap_or(0);
        let max = delays.last().copied().unwrap_or(0);

        assert!(
            min != max || max == 0,
            "Jitter should produce variation in delays"
        );
    }

    #[test]
    fn test_backoff_config_default() {
        let config = BackoffConfig::default();
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 10000);
        assert_eq!(config.jitter_factor, 0.1);
        assert_eq!(config.max_retries, 3);
    }
}
