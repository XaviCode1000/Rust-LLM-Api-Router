//! Prometheus metrics

use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Registry};
use std::sync::Arc;

use crate::app::services::execution_plan::ExecutionPlanMetrics;

/// Combined metrics for the application.
pub struct Metrics {
    pub registry: Registry,
    pub requests_total: Counter,
    pub requests_in_flight: Gauge,
    pub request_duration: Histogram,
    pub execution_plan: Option<ExecutionPlanMetrics>,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let requests_total = Counter::new("llm_requests_total", "Total number of LLM requests")?;

        let requests_in_flight = Gauge::new(
            "llm_requests_in_flight",
            "Number of requests currently being processed",
        )?;

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "llm_request_duration_seconds",
                "Request duration in seconds",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0]),
        )?;

        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(requests_in_flight.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;

        Ok(Self {
            registry,
            requests_total,
            requests_in_flight,
            request_duration,
            execution_plan: None,
        })
    }

    /// Creates metrics with execution plan metrics included.
    pub fn with_execution_plan() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let requests_total = Counter::new("llm_requests_total", "Total number of LLM requests")?;

        let requests_in_flight = Gauge::new(
            "llm_requests_in_flight",
            "Number of requests currently being processed",
        )?;

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "llm_request_duration_seconds",
                "Request duration in seconds",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0]),
        )?;

        // Create execution plan metrics
        let execution_plan = ExecutionPlanMetrics::new(&registry)?;

        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(requests_in_flight.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;

        Ok(Self {
            registry,
            requests_total,
            requests_in_flight,
            request_duration,
            execution_plan: Some(execution_plan),
        })
    }

    /// Gets the execution plan metrics if available.
    pub fn execution_plan_metrics(&self) -> Option<&ExecutionPlanMetrics> {
        self.execution_plan.as_ref()
    }
}

pub type SharedMetrics = Arc<Metrics>;
