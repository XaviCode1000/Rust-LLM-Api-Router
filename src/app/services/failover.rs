//! Failover logic for account rotation
//!
//! This module provides automatic failover between accounts when requests fail.

use std::sync::Arc;

use crate::domain::traits::AccountRepository;
use crate::domain::{Account, AccountHealth};
use crate::infrastructure::JsonAccountRepository;

use super::account_rotation::AccountSelector;

/// Failover manager for account requests.
///
/// Handles automatic retry with different accounts when requests fail.
pub struct FailoverManager {
    /// Account repository for fetching accounts
    account_repo: Arc<JsonAccountRepository>,

    /// Account selector with rotation strategy
    selector: AccountSelector,

    /// Health tracking per account
    health_map: std::sync::Mutex<std::collections::HashMap<String, AccountHealth>>,

    /// Maximum retry attempts
    max_retries: u32,
}

impl FailoverManager {
    /// Creates a new failover manager.
    pub fn new(
        account_repo: Arc<JsonAccountRepository>,
        selector: AccountSelector,
        max_retries: u32,
    ) -> Self {
        Self {
            account_repo,
            selector,
            health_map: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_retries,
        }
    }

    /// Creates a new failover manager with round-robin strategy.
    pub fn with_round_robin(account_repo: Arc<JsonAccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::round_robin(), 3)
    }

    /// Creates a new failover manager with weighted strategy.
    pub fn with_weighted(account_repo: Arc<JsonAccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::weighted(), 3)
    }

    /// Creates a new failover manager with latency-based strategy.
    pub fn with_latency_based(account_repo: Arc<JsonAccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::latency_based(), 3)
    }

    /// Creates a new failover manager with user-affinity strategy.
    pub fn with_user_affinity(account_repo: Arc<JsonAccountRepository>) -> Self {
        Self::new(account_repo, AccountSelector::user_affinity(), 3)
    }

    /// Executes a request with automatic failover.
    ///
    /// # Arguments
    /// * `provider_id` - Provider ID to get accounts for
    /// * `execute` - Async function to execute the request with an account
    ///
    /// # Returns
    /// The response if successful, or the last error if all accounts fail
    pub async fn execute_with_failover<F, Fut, T, E>(
        &self,
        provider_id: &str,
        execute: F,
    ) -> Result<T, E>
    where
        F: Fn(&Account) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
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
                    Ok(response) => {
                        // Record success
                        self.record_success(&account.id, start.elapsed().as_millis() as u64);
                        return Ok(response);
                    }
                    Err(e) => {
                        // Record failure
                        self.record_failure(&account.id);
                        last_error = Some(e);

                        // Continue to next account if we have retries left
                        if attempt < self.max_retries as usize - 1 {
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
