//! Execution Plan
//!
//! Defines the core trait for execution plans and related types.

use serde::{Deserialize, Serialize};

use super::types::{ExecutionPlanType, PlannedAccount};
use super::{ExecutionContext, ExecutionOutcome, ExecutionPlanStatus};

/// Trait for execution plans.
///
/// An execution plan defines how an LLM request should be executed,
/// including which accounts to use, retry strategies, and failover handling.
pub trait ExecutionPlan: Send + Sync {
    /// Returns the type of this execution plan.
    fn plan_type(&self) -> ExecutionPlanType;

    /// Returns the planned accounts for this execution.
    fn planned_accounts(&self) -> &[PlannedAccount];

    /// Returns the current status of the plan.
    fn status(&self) -> ExecutionPlanStatus;

    /// Returns the context that generated this plan.
    fn context(&self) -> &ExecutionContext;

    /// Returns the max number of retries allowed.
    fn max_retries(&self) -> u32;

    /// Returns the timeout in seconds for this execution.
    fn timeout_seconds(&self) -> u32;

    /// Returns the outcome of the execution (if completed).
    fn outcome(&self) -> Option<ExecutionOutcome>;

    /// Returns an error message if the plan failed.
    fn error_message(&self) -> Option<&str>;

    /// Returns the number of accounts in this plan.
    fn account_count(&self) -> usize {
        self.planned_accounts().len()
    }

    /// Returns true if the plan has accounts to execute with.
    fn has_accounts(&self) -> bool {
        !self.planned_accounts().is_empty()
    }

    /// Returns the primary account (first non-fallback).
    fn primary_account(&self) -> Option<&PlannedAccount> {
        self.planned_accounts()
            .iter()
            .find(|a| a.is_primary)
            .or_else(|| self.planned_accounts().first())
    }

    /// Returns fallback accounts.
    fn fallback_accounts(&self) -> Vec<&PlannedAccount> {
        self.planned_accounts()
            .iter()
            .filter(|a| a.is_fallback)
            .collect()
    }

    /// Checks if this plan supports failover.
    fn supports_failover(&self) -> bool {
        self.plan_type().supports_failover() && self.account_count() > 1
    }

    /// Returns the next account to try based on execution order.
    fn next_account(&self, failed_account_id: Option<&str>) -> Option<&PlannedAccount> {
        let accounts = self.planned_accounts();

        if let Some(failed_id) = failed_account_id {
            // Find the failed account and return the next one
            if let Some(failed_idx) = accounts.iter().position(|a| a.account_id == failed_id) {
                return accounts.get(failed_idx + 1);
            }
        }

        // Return first account if no failure specified
        accounts.first()
    }

    /// Updates the plan status.
    fn update_status(&mut self, status: ExecutionPlanStatus);

    /// Sets the outcome of the execution.
    fn set_outcome(&mut self, outcome: ExecutionOutcome);

    /// Sets an error message.
    fn set_error(&mut self, message: impl Into<String>);
}

/// A concrete execution plan implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlanImpl {
    /// Type of execution plan
    plan_type: ExecutionPlanType,

    /// Planned accounts
    planned_accounts: Vec<PlannedAccount>,

    /// Current status
    status: ExecutionPlanStatus,

    /// Context that generated this plan
    context: ExecutionContext,

    /// Max retries
    max_retries: u32,

    /// Timeout in seconds
    timeout_seconds: u32,

    /// Outcome (if completed)
    outcome: Option<ExecutionOutcome>,

    /// Error message (if failed)
    error_message: Option<String>,
}

impl ExecutionPlanImpl {
    /// Creates a new `ExecutionPlanImpl`.
    pub fn new(
        plan_type: ExecutionPlanType,
        context: ExecutionContext,
        planned_accounts: Vec<PlannedAccount>,
    ) -> Self {
        Self {
            plan_type,
            planned_accounts,
            status: ExecutionPlanStatus::Planned,
            context,
            max_retries: 3,
            timeout_seconds: 60,
            outcome: None,
            error_message: None,
        }
    }

    /// Sets max retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Sets timeout.
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Gets a mutable reference to planned accounts.
    pub fn planned_accounts_mut(&mut self) -> &mut Vec<PlannedAccount> {
        &mut self.planned_accounts
    }
}

impl ExecutionPlan for ExecutionPlanImpl {
    fn plan_type(&self) -> ExecutionPlanType {
        self.plan_type
    }

    fn planned_accounts(&self) -> &[PlannedAccount] {
        &self.planned_accounts
    }

    fn status(&self) -> ExecutionPlanStatus {
        self.status
    }

    fn context(&self) -> &ExecutionContext {
        &self.context
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }

    fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.outcome
    }

    fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.status = status;
    }

    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.outcome = Some(outcome);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }
}

/// Type alias for boxed execution plans.
pub type BoxedExecutionPlan = Box<dyn ExecutionPlan>;

/// Builder for creating execution plans.
#[derive(Debug, Default)]
pub struct ExecutionPlanBuilder {
    plan_type: Option<ExecutionPlanType>,
    context: Option<ExecutionContext>,
    planned_accounts: Vec<PlannedAccount>,
    max_retries: Option<u32>,
    timeout_seconds: Option<u32>,
}

impl ExecutionPlanBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the plan type.
    pub fn with_plan_type(mut self, plan_type: ExecutionPlanType) -> Self {
        self.plan_type = Some(plan_type);
        self
    }

    /// Sets the execution context.
    pub fn with_context(mut self, context: ExecutionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Adds a planned account.
    pub fn with_account(mut self, account: PlannedAccount) -> Self {
        self.planned_accounts.push(account);
        self
    }

    /// Adds multiple planned accounts.
    pub fn with_accounts(mut self, accounts: Vec<PlannedAccount>) -> Self {
        self.planned_accounts.extend(accounts);
        self
    }

    /// Sets max retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Sets timeout.
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Builds the execution plan.
    ///
    /// # Panics
    ///
    /// Panics if required fields are missing.
    pub fn build(self) -> ExecutionPlanImpl {
        let context = self.context.expect("ExecutionContext is required");
        let plan_type = self.plan_type.unwrap_or_default();

        let max_retries = self
            .max_retries
            .unwrap_or(context.planning_options.max_retries);
        let timeout_seconds = self
            .timeout_seconds
            .unwrap_or(context.planning_options.timeout_seconds);

        ExecutionPlanImpl::new(plan_type, context, self.planned_accounts)
            .with_max_retries(max_retries)
            .with_timeout(timeout_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{AccountHealth, Provider};

    fn create_test_context() -> ExecutionContext {
        ExecutionContext::new("test-req-1", "gpt-4")
    }

    fn create_test_plan() -> ExecutionPlanImpl {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health = AccountHealth::new("acc-1");
        let planned = PlannedAccount::new("acc-1", &provider, health);

        ExecutionPlanImpl::new(
            ExecutionPlanType::Standard,
            create_test_context(),
            vec![planned],
        )
    }

    #[test]
    fn test_execution_plan_status() {
        let mut plan = create_test_plan();
        assert_eq!(plan.status(), ExecutionPlanStatus::Planned);

        plan.update_status(ExecutionPlanStatus::InProgress);
        assert_eq!(plan.status(), ExecutionPlanStatus::InProgress);
    }

    #[test]
    fn test_execution_plan_outcome() {
        let mut plan = create_test_plan();
        assert!(plan.outcome().is_none());

        plan.set_outcome(ExecutionOutcome::Success);
        assert_eq!(plan.outcome(), Some(ExecutionOutcome::Success));
    }

    #[test]
    fn test_execution_plan_primary_account() {
        let plan = create_test_plan();
        let primary = plan.primary_account().expect("Should have primary");
        assert_eq!(primary.account_id, "acc-1");
    }

    #[test]
    fn test_execution_plan_builder() {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health = AccountHealth::new("acc-1");
        let account = PlannedAccount::new("acc-1", &provider, health);

        let plan = ExecutionPlanBuilder::new()
            .with_plan_type(ExecutionPlanType::Failover)
            .with_context(create_test_context())
            .with_account(account)
            .with_max_retries(5)
            .with_timeout(30)
            .build();

        assert_eq!(plan.plan_type(), ExecutionPlanType::Failover);
        assert_eq!(plan.max_retries(), 5);
        assert_eq!(plan.timeout_seconds(), 30);
    }

    #[test]
    fn test_execution_plan_next_account() {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health1 = AccountHealth::new("acc-1");
        let health2 = AccountHealth::new("acc-2");

        let account1 = PlannedAccount::new("acc-1", &provider, health1);
        let account2 = PlannedAccount::new("acc-2", &provider, health2).as_fallback();

        let mut plan = ExecutionPlanBuilder::new()
            .with_plan_type(ExecutionPlanType::Failover)
            .with_context(create_test_context())
            .with_accounts(vec![account1, account2])
            .build();

        // First call returns first account
        let next = plan.next_account(None);
        assert_eq!(next.unwrap().account_id, "acc-1");

        // After failure, returns second account
        let next = plan.next_account(Some("acc-1"));
        assert_eq!(next.unwrap().account_id, "acc-2");
    }
}
