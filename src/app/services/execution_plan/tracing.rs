//! Execution Plan Tracing
//!
//! Provides tracing spans and audit logging for execution plan creation
//! and execution.

use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Decision log entry for audit trail.
#[derive(Debug, Clone)]
pub struct DecisionLogEntry {
    /// Timestamp of the decision
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Request ID for correlation
    pub request_id: String,

    /// The decision that was made
    pub decision: String,

    /// Reason for the decision
    pub reason: String,

    /// The options that were considered
    pub options_considered: Vec<String>,

    /// Selected option
    pub selected: String,

    /// Additional metadata
    pub metadata: Vec<(String, String)>,
}

impl DecisionLogEntry {
    /// Creates a new decision log entry.
    pub fn new(request_id: impl Into<String>, decision: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            request_id: request_id.into(),
            decision: decision.into(),
            reason: String::new(),
            options_considered: Vec::new(),
            selected: String::new(),
            metadata: Vec::new(),
        }
    }

    /// Sets the reason.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Sets the options considered.
    pub fn with_options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options_considered = options.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Sets the selected option.
    pub fn with_selected(mut self, selected: impl Into<String>) -> Self {
        self.selected = selected.into();
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Logs this entry at the INFO level.
    pub fn log(&self) {
        info!(
            request_id = %self.request_id,
            decision = %self.decision,
            reason = %self.reason,
            options = ?self.options_considered,
            selected = %self.selected,
            metadata = ?self.metadata,
            "Execution plan decision"
        );
    }
}

/// Builder for creating decision log entries.
#[derive(Debug)]
pub struct DecisionLogBuilder {
    request_id: String,
    decision: String,
    reason: Option<String>,
    options: Vec<String>,
    selected: Option<String>,
    metadata: Vec<(String, String)>,
}

impl DecisionLogBuilder {
    /// Creates a new builder.
    pub fn new(request_id: impl Into<String>, decision: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            decision: decision.into(),
            reason: None,
            options: Vec::new(),
            selected: None,
            metadata: Vec::new(),
        }
    }

    /// Sets the reason.
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds options considered.
    pub fn options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Adds a single option.
    pub fn option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Sets the selected option.
    pub fn selected(mut self, selected: impl Into<String>) -> Self {
        self.selected = Some(selected.into());
        self
    }

    /// Adds metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Builds and logs the decision.
    pub fn log(self) {
        let entry = DecisionLogEntry {
            timestamp: chrono::Utc::now(),
            request_id: self.request_id,
            decision: self.decision,
            reason: self.reason.unwrap_or_default(),
            options_considered: self.options,
            selected: self.selected.unwrap_or_default(),
            metadata: self.metadata,
        };
        entry.log();
    }

    /// Builds the entry without logging.
    pub fn build(self) -> DecisionLogEntry {
        DecisionLogEntry {
            timestamp: chrono::Utc::now(),
            request_id: self.request_id,
            decision: self.decision,
            reason: self.reason.unwrap_or_default(),
            options_considered: self.options,
            selected: self.selected.unwrap_or_default(),
            metadata: self.metadata,
        }
    }
}

/// Span builder for execution planning.
#[derive(Debug)]
pub struct PlanningSpan {
    request_id: String,
    model: String,
    start_time: Instant,
}

impl PlanningSpan {
    /// Creates a new planning span.
    pub fn new(request_id: impl Into<String>, model: impl Into<String>) -> Self {
        let request_id = request_id.into();
        let model = model.into();

        debug!(request_id = %request_id, model = %model, "Starting execution plan creation");

        Self {
            request_id,
            model,
            start_time: Instant::now(),
        }
    }

    /// Records plan type selection.
    pub fn record_plan_type(&self, plan_type: &str) {
        debug!(
            request_id = %self.request_id,
            plan_type = %plan_type,
            "Selected execution plan type"
        );
    }

    /// Records accounts selected.
    pub fn record_accounts(&self, account_count: usize, primary_account: Option<&str>) {
        debug!(
            request_id = %self.request_id,
            account_count = account_count,
            primary_account = ?primary_account,
            "Accounts selected for execution plan"
        );
    }

    /// Records a filter applied.
    pub fn record_filter(&self, filter_name: &str, remaining_count: usize) {
        debug!(
            request_id = %self.request_id,
            filter = %filter_name,
            remaining_accounts = remaining_count,
            "Applied filter to accounts"
        );
    }

    /// Completes the span.
    pub fn finish(&self, plan_type: &str, account_count: usize) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        info!(
            request_id = %self.request_id,
            model = %self.model,
            plan_type = %plan_type,
            account_count = account_count,
            duration_ms = duration_ms,
            "Execution plan created successfully"
        );
    }

    /// Records an error during planning.
    pub fn error(&self, error: &str) {
        error!(
            request_id = %self.request_id,
            error = %error,
            "Error creating execution plan"
        );
    }
}

/// Span builder for execution.
#[derive(Debug)]
pub struct ExecutionSpan {
    request_id: String,
    account_id: String,
    attempt: u32,
    start_time: Instant,
}

impl ExecutionSpan {
    /// Creates a new execution span.
    pub fn new(request_id: impl Into<String>, account_id: impl Into<String>, attempt: u32) -> Self {
        let request_id = request_id.into();
        let account_id = account_id.into();

        debug!(
            request_id = %request_id,
            account_id = %account_id,
            attempt = attempt,
            "Starting execution attempt"
        );

        Self {
            request_id,
            account_id,
            attempt,
            start_time: Instant::now(),
        }
    }

    /// Records success.
    pub fn success(&self) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        info!(
            request_id = %self.request_id,
            account_id = %self.account_id,
            attempt = self.attempt,
            duration_ms = duration_ms,
            "Execution succeeded"
        );
    }

    /// Records fallback to next account.
    pub fn fallback(&self, next_account_id: Option<&str>) {
        warn!(
            request_id = %self.request_id,
            account_id = %self.account_id,
            attempt = self.attempt,
            next_account = ?next_account_id,
            "Falling back to next account"
        );
    }

    /// Records a failure.
    pub fn failure(&self, error: &str) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        error!(
            request_id = %self.request_id,
            account_id = %self.account_id,
            attempt = self.attempt,
            duration_ms = duration_ms,
            error = %error,
            "Execution failed"
        );
    }
}

/// Logging utilities for execution plans.
pub mod logging {
    use super::*;

    /// Logs a plan type selection decision.
    pub fn log_plan_type_selection(request_id: &str, context: &str, plan_type: &str, reason: &str) {
        DecisionLogBuilder::new(request_id, "plan_type_selection")
            .reason(reason)
            .option("Standard")
            .option("Failover")
            .option("LoadBalanced")
            .option("CostOptimized")
            .selected(plan_type)
            .metadata("context", context)
            .log();
    }

    /// Logs account filtering decision.
    pub fn log_account_filtering(
        request_id: &str,
        filter_name: &str,
        initial_count: usize,
        remaining_count: usize,
    ) {
        debug!(
            request_id = %request_id,
            filter = %filter_name,
            initial_accounts = initial_count,
            remaining_accounts = remaining_count,
            "Filtered accounts"
        );
    }

    /// Logs rotation strategy applied.
    pub fn log_rotation_strategy(request_id: &str, strategy: &str, account_count: usize) {
        DecisionLogBuilder::new(request_id, "rotation_strategy")
            .reason(format!("Applied {} strategy", strategy))
            .option("RoundRobin")
            .option("HealthWeighted")
            .option("Priority")
            .option("LeastRecentlyUsed")
            .selected(strategy)
            .metadata("account_count", account_count.to_string())
            .log();
    }

    /// Logs fallback account selection.
    #[allow(dead_code)]
    pub fn log_fallback_selection(request_id: &str, from_account: &str, to_account: &str) {
        DecisionLogBuilder::new(request_id, "fallback_selection")
            .reason(format!("Primary account {} failed", from_account))
            .selected(to_account)
            .metadata("failed_account", from_account)
            .log();
    }

    /// Logs execution outcome.
    #[allow(dead_code)]
    pub fn log_execution_outcome(
        request_id: &str,
        outcome: &str,
        account_used: &str,
        duration_ms: f64,
    ) {
        info!(
            request_id = %request_id,
            outcome = %outcome,
            account_id = %account_used,
            duration_ms = duration_ms,
            "Execution completed"
        );
    }
}

/// Span builder for quality evaluation.
#[derive(Debug)]
pub struct QualityEvaluationSpan {
    request_id: String,
    account_id: String,
    tier_index: u32,
    start_time: Instant,
}

impl QualityEvaluationSpan {
    /// Creates a new quality evaluation span.
    pub fn new(
        request_id: impl Into<String>,
        account_id: impl Into<String>,
        tier_index: u32,
    ) -> Self {
        let request_id = request_id.into();
        let account_id = account_id.into();

        debug!(
            request_id = %request_id,
            account_id = %account_id,
            tier_index = tier_index,
            "Starting quality evaluation"
        );

        Self {
            request_id,
            account_id,
            tier_index,
            start_time: Instant::now(),
        }
    }

    /// Records an individual check result.
    pub fn record_check(&self, check_name: &str, passed: bool) {
        if passed {
            debug!(
                request_id = %self.request_id,
                tier_index = self.tier_index,
                check = %check_name,
                "Quality check passed"
            );
        } else {
            debug!(
                request_id = %self.request_id,
                tier_index = self.tier_index,
                check = %check_name,
                "Quality check failed"
            );
        }
    }

    /// Completes the span with final score.
    pub fn finish(&self, score: f64, is_acceptable: bool, checks_failed: &[String]) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        if is_acceptable {
            info!(
                request_id = %self.request_id,
                account_id = %self.account_id,
                tier_index = self.tier_index,
                score = score,
                checks_failed = ?checks_failed,
                checks_passed = 4usize.saturating_sub(checks_failed.len()),
                duration_ms = duration_ms,
                "Quality evaluation passed"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                account_id = %self.account_id,
                tier_index = self.tier_index,
                score = score,
                checks_failed = ?checks_failed,
                duration_ms = duration_ms,
                "Quality evaluation failed — escalation triggered"
            );
        }
    }
}

/// Span builder for quality evaluation.
#[derive(Debug)]
pub struct QualityEvaluationSpan {
    request_id: String,
    account_id: String,
    tier_index: u32,
    start_time: Instant,
}

impl QualityEvaluationSpan {
    /// Creates a new quality evaluation span.
    pub fn new(
        request_id: impl Into<String>,
        account_id: impl Into<String>,
        tier_index: u32,
    ) -> Self {
        let request_id = request_id.into();
        let account_id = account_id.into();

        debug!(
            request_id = %request_id,
            account_id = %account_id,
            tier_index = tier_index,
            "Starting quality evaluation"
        );

        Self {
            request_id,
            account_id,
            tier_index,
            start_time: Instant::now(),
        }
    }

    /// Records an individual check result.
    pub fn record_check(&self, check_name: &str, passed: bool) {
        if passed {
            debug!(
                request_id = %self.request_id,
                tier_index = self.tier_index,
                check = %check_name,
                "Quality check passed"
            );
        } else {
            debug!(
                request_id = %self.request_id,
                tier_index = self.tier_index,
                check = %check_name,
                "Quality check failed"
            );
        }
    }

    /// Completes the span with final score.
    pub fn finish(&self, score: f64, is_acceptable: bool, checks_failed: &[String]) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        if is_acceptable {
            info!(
                request_id = %self.request_id,
                account_id = %self.account_id,
                tier_index = self.tier_index,
                score = score,
                checks_failed = ?checks_failed,
                checks_passed = 4usize.saturating_sub(checks_failed.len()),
                duration_ms = duration_ms,
                "Quality evaluation passed"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                account_id = %self.account_id,
                tier_index = self.tier_index,
                score = score,
                checks_failed = ?checks_failed,
                duration_ms = duration_ms,
                "Quality evaluation failed — escalation triggered"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_log_entry() {
        let entry = DecisionLogEntry::new("req-1", "test_decision")
            .with_reason("Testing")
            .with_options(vec!["option1", "option2"])
            .with_selected("option1")
            .with_metadata("key", "value");

        assert_eq!(entry.request_id, "req-1");
        assert_eq!(entry.decision, "test_decision");
        assert_eq!(entry.reason, "Testing");
        assert_eq!(entry.options_considered.len(), 2);
        assert_eq!(entry.selected, "option1");
        assert_eq!(entry.metadata.len(), 1);
    }

    #[test]
    fn test_decision_log_builder() {
        let builder = DecisionLogBuilder::new("req-1", "test")
            .reason("Testing the builder")
            .option("opt1")
            .option("opt2")
            .selected("opt1")
            .metadata("meta", "data");

        let entry = builder.build();

        assert_eq!(entry.request_id, "req-1");
        assert_eq!(entry.decision, "test");
    }

    #[test]
    fn test_planning_span() {
        let span = PlanningSpan::new("req-1", "gpt-4");
        span.record_plan_type("Failover");
        span.record_accounts(3, Some("acc-1"));
        span.finish("Failover", 3);
    }

    #[test]
    fn test_execution_span() {
        let span = ExecutionSpan::new("req-1", "acc-1", 1);
        span.success();

        let span2 = ExecutionSpan::new("req-1", "acc-1", 1);
        span2.fallback(Some("acc-2"));

        let span3 = ExecutionSpan::new("req-1", "acc-1", 1);
        span3.failure("API error");
    }
}
