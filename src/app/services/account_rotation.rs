//! Account rotation strategies
//!
//! This module provides different strategies for selecting which account
//! to use when making requests to LLM providers.

use crate::domain::Account;

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

        // For now, just return the first account
        // TODO: Integrate with AccountHealth to select based on latency
        accounts.first()
    }

    fn name(&self) -> &str {
        "latency-based"
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
}

/// Account selector that uses a rotation strategy.
pub struct AccountSelector {
    strategy: Box<dyn RotationStrategy>,
}

impl AccountSelector {
    /// Creates a new selector with the specified strategy.
    pub fn with_strategy(strategy: Box<dyn RotationStrategy>) -> Self {
        Self { strategy }
    }

    /// Creates a new selector with round-robin strategy.
    pub fn round_robin() -> Self {
        Self {
            strategy: Box::new(RoundRobinStrategy::new()),
        }
    }

    /// Creates a new selector with weighted strategy.
    pub fn weighted() -> Self {
        Self {
            strategy: Box::new(WeightedStrategy::new()),
        }
    }

    /// Creates a new selector with latency-based strategy.
    pub fn latency_based() -> Self {
        Self {
            strategy: Box::new(LatencyStrategy::new()),
        }
    }

    /// Creates a new selector with user-affinity strategy.
    pub fn user_affinity() -> Self {
        Self {
            strategy: Box::new(UserAffinityStrategy::new()),
        }
    }

    /// Selects an account from the list using the configured strategy.
    pub fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        self.strategy.select(accounts)
    }

    /// Returns the current strategy name.
    pub fn strategy_name(&self) -> &str {
        self.strategy.name()
    }
}
