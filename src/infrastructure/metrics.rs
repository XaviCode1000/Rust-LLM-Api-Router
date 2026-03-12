//! Prometheus metrics

use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Registry};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,
    pub requests_total: Counter,
    pub requests_in_flight: Gauge,
    pub request_duration: Histogram,
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
        })
    }
}

pub type SharedMetrics = Arc<Metrics>;
