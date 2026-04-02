//! Execution Plan Metrics
//!
//! Provides metrics for monitoring execution plan creation, planning time,
//! plan type distribution, and fallback usage.

use prometheus::{Histogram, HistogramOpts, IntCounter, IntGauge, Registry};

/// Metrics for execution planning.
#[derive(Debug, Clone)]
pub struct ExecutionPlanMetrics {
    /// Total number of plans created
    pub plans_created_total: IntCounter,

    /// Current number of plans being executed
    pub plans_in_flight: IntGauge,

    /// Histogram of planning duration
    pub planning_duration_seconds: Histogram,

    /// Counter for plan types - standard
    pub plan_type_standard_total: IntCounter,

    /// Counter for plan types - failover
    pub plan_type_failover_total: IntCounter,

    /// Counter for plan types - load balanced
    pub plan_type_load_balanced_total: IntCounter,

    /// Counter for plan types - cost optimized
    pub plan_type_cost_optimized_total: IntCounter,

    /// Counter for fallback usage
    pub fallback_usage_total: IntCounter,

    /// Counter for plan outcomes - success
    pub plan_outcome_success_total: IntCounter,

    /// Counter for plan outcomes - failure
    pub plan_outcome_failure_total: IntCounter,

    /// Counter for plan outcomes - fallback success
    pub plan_outcome_fallback_total: IntCounter,

    /// Counter for planning errors
    pub planning_errors_total: IntCounter,

    /// Gauge for current planning attempts
    pub planning_attempts_current: IntGauge,
}

impl ExecutionPlanMetrics {
    /// Creates new execution plan metrics.
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let plans_created_total = IntCounter::new(
            "execution_plans_created_total",
            "Total number of execution plans created",
        )?;

        let plans_in_flight = IntGauge::new(
            "execution_plans_in_flight",
            "Number of execution plans currently being executed",
        )?;

        let planning_duration_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "execution_planning_duration_seconds",
                "Time taken to create an execution plan in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
        )?;

        let plan_type_standard_total = IntCounter::new(
            "execution_plan_type_standard_total",
            "Number of standard execution plans created",
        )?;

        let plan_type_failover_total = IntCounter::new(
            "execution_plan_type_failover_total",
            "Number of failover execution plans created",
        )?;

        let plan_type_load_balanced_total = IntCounter::new(
            "execution_plan_type_load_balanced_total",
            "Number of load balanced execution plans created",
        )?;

        let plan_type_cost_optimized_total = IntCounter::new(
            "execution_plan_type_cost_optimized_total",
            "Number of cost optimized execution plans created",
        )?;

        let fallback_usage_total = IntCounter::new(
            "execution_plan_fallback_usage_total",
            "Total number of times a fallback account was used",
        )?;

        let plan_outcome_success_total = IntCounter::new(
            "execution_plan_outcome_success_total",
            "Number of successful executions",
        )?;

        let plan_outcome_failure_total =
            IntCounter::new("execution_plan_outcome_failure_total", "Number of failed executions")?;

        let plan_outcome_fallback_total = IntCounter::new(
            "execution_plan_outcome_fallback_total",
            "Number of executions that succeeded with fallback",
        )?;

        let planning_errors_total =
            IntCounter::new("execution_planning_errors_total", "Total number of planning errors")?;

        let planning_attempts_current = IntGauge::new(
            "execution_planning_attempts_current",
            "Current number of planning attempts in progress",
        )?;

        // Register all metrics
        registry.register(Box::new(plans_created_total.clone()))?;
        registry.register(Box::new(plans_in_flight.clone()))?;
        registry.register(Box::new(planning_duration_seconds.clone()))?;
        registry.register(Box::new(plan_type_standard_total.clone()))?;
        registry.register(Box::new(plan_type_failover_total.clone()))?;
        registry.register(Box::new(plan_type_load_balanced_total.clone()))?;
        registry.register(Box::new(plan_type_cost_optimized_total.clone()))?;
        registry.register(Box::new(fallback_usage_total.clone()))?;
        registry.register(Box::new(plan_outcome_success_total.clone()))?;
        registry.register(Box::new(plan_outcome_failure_total.clone()))?;
        registry.register(Box::new(plan_outcome_fallback_total.clone()))?;
        registry.register(Box::new(planning_errors_total.clone()))?;
        registry.register(Box::new(planning_attempts_current.clone()))?;

        Ok(Self {
            plans_created_total,
            plans_in_flight,
            planning_duration_seconds,
            plan_type_standard_total,
            plan_type_failover_total,
            plan_type_load_balanced_total,
            plan_type_cost_optimized_total,
            fallback_usage_total,
            plan_outcome_success_total,
            plan_outcome_failure_total,
            plan_outcome_fallback_total,
            planning_errors_total,
            planning_attempts_current,
        })
    }

    /// Records a plan creation.
    pub fn record_plan_created(&self) {
        self.plans_created_total.inc();
    }

    /// Records a plan starting execution.
    pub fn record_plan_started(&self) {
        self.plans_in_flight.inc();
    }

    /// Records a plan completing execution.
    pub fn record_plan_completed(&self) {
        self.plans_in_flight.dec();
    }

    /// Records planning duration.
    pub fn record_planning_duration(&self, duration_secs: f64) {
        self.planning_duration_seconds.observe(duration_secs);
    }

    /// Records a plan type being used.
    pub fn record_plan_type(&self, plan_type: &str) {
        match plan_type {
            "Standard" => self.plan_type_standard_total.inc(),
            "Failover" => self.plan_type_failover_total.inc(),
            "Load Balanced" => self.plan_type_load_balanced_total.inc(),
            "Cost Optimized" => self.plan_type_cost_optimized_total.inc(),
            _ => {}, // Ignore unknown plan types
        }
    }

    /// Records fallback usage.
    pub fn record_fallback_used(&self) {
        self.fallback_usage_total.inc();
    }

    /// Records a plan outcome.
    pub fn record_outcome(&self, outcome: &str) {
        match outcome {
            "success" => self.plan_outcome_success_total.inc(),
            "failure" => self.plan_outcome_failure_total.inc(),
            "fallback" => self.plan_outcome_fallback_total.inc(),
            _ => {}, // Ignore unknown outcomes
        }
    }

    /// Records a planning error.
    pub fn record_planning_error(&self) {
        self.planning_errors_total.inc();
    }

    /// Records a planning attempt starting.
    pub fn record_planning_started(&self) {
        self.planning_attempts_current.inc();
    }

    /// Records a planning attempt completing.
    pub fn record_planning_completed(&self) {
        self.planning_attempts_current.dec();
    }
}

/// Builder for creating ExecutionPlanMetrics with common labels.
#[derive(Debug, Clone)]
pub struct ExecutionPlanMetricsBuilder {
    labels: Vec<(&'static str, String)>,
}

impl ExecutionPlanMetricsBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    /// Adds a label.
    pub fn with_label(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.labels.push((key, value.into()));
        self
    }

    /// Builds the metrics (labels are ignored in this implementation,
    /// but could be used for more advanced metric configurations).
    pub fn build(self) -> Self {
        self
    }
}

impl Default for ExecutionPlanMetricsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;

    #[test]
    fn test_metrics_labels_builder() {
        let builder = ExecutionPlanMetricsBuilder::new()
            .with_label("service", "execution-planner")
            .with_label("version", "1.0.0");

        let _built = builder.build();
    }

    #[test]
    fn test_metrics_builder_default() {
        let builder = ExecutionPlanMetricsBuilder::default();
        let _built = builder.build();
    }

    #[test]
    fn test_execution_plan_metrics_creation() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry);

        assert!(metrics.is_ok());
    }

    #[test]
    fn test_record_plan_created() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_plan_created();
        metrics.record_plan_created();

        assert_eq!(metrics.plans_created_total.get(), 2);
    }

    #[test]
    fn test_record_plan_started_completed() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_plan_started();
        assert_eq!(metrics.plans_in_flight.get(), 1);

        metrics.record_plan_started();
        assert_eq!(metrics.plans_in_flight.get(), 2);

        metrics.record_plan_completed();
        assert_eq!(metrics.plans_in_flight.get(), 1);

        metrics.record_plan_completed();
        assert_eq!(metrics.plans_in_flight.get(), 0);
    }

    #[test]
    fn test_record_planning_duration() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_planning_duration(0.001);
        metrics.record_planning_duration(0.005);
        metrics.record_planning_duration(0.01);

        // Histogram should have recorded these values - just verify no panic
        // The histogram is working internally
        let _ = metrics.planning_duration_seconds;
    }

    #[test]
    fn test_record_plan_type() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_plan_type("Standard");
        metrics.record_plan_type("Standard");
        metrics.record_plan_type("Failover");

        assert_eq!(metrics.plan_type_standard_total.get(), 2);
        assert_eq!(metrics.plan_type_failover_total.get(), 1);
        assert_eq!(metrics.plan_type_load_balanced_total.get(), 0);
        assert_eq!(metrics.plan_type_cost_optimized_total.get(), 0);
    }

    #[test]
    fn test_record_fallback_used() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_fallback_used();
        metrics.record_fallback_used();
        metrics.record_fallback_used();

        assert_eq!(metrics.fallback_usage_total.get(), 3);
    }

    #[test]
    fn test_record_outcome() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_outcome("success");
        metrics.record_outcome("success");
        metrics.record_outcome("failure");
        metrics.record_outcome("fallback");

        assert_eq!(metrics.plan_outcome_success_total.get(), 2);
        assert_eq!(metrics.plan_outcome_failure_total.get(), 1);
        assert_eq!(metrics.plan_outcome_fallback_total.get(), 1);
    }

    #[test]
    fn test_record_planning_error() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_planning_error();

        assert_eq!(metrics.planning_errors_total.get(), 1);
    }

    #[test]
    fn test_record_planning_attempts() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        metrics.record_planning_started();
        assert_eq!(metrics.planning_attempts_current.get(), 1);

        metrics.record_planning_started();
        assert_eq!(metrics.planning_attempts_current.get(), 2);

        metrics.record_planning_completed();
        assert_eq!(metrics.planning_attempts_current.get(), 1);

        metrics.record_planning_completed();
        assert_eq!(metrics.planning_attempts_current.get(), 0);
    }

    #[test]
    fn test_record_unknown_plan_type() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        // Unknown plan types should not panic
        metrics.record_plan_type("Unknown");

        // All counters should remain at 0
        assert_eq!(metrics.plan_type_standard_total.get(), 0);
    }

    #[test]
    fn test_record_unknown_outcome() {
        let registry = Registry::new();
        let metrics = ExecutionPlanMetrics::new(&registry).unwrap();

        // Unknown outcomes should not panic
        metrics.record_outcome("unknown");

        // All counters should remain at 0
        assert_eq!(metrics.plan_outcome_success_total.get(), 0);
    }
}
