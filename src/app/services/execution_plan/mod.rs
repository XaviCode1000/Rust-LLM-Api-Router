//! Execution Plan Module
//!
//! This module provides the foundational types for planning and executing LLM requests.
//! It defines the execution context, plan types, and the core trait for execution strategies.

mod context;
mod implementations;
mod metrics;
mod outcome;
mod plan;
mod planner;
mod status;
mod tracing;
mod types;

pub use context::{ExecutionContext, PlanningOptions};
pub use implementations::{
    CostOptimizedExecutionPlan, ExecutionPlanBuilder, FailoverExecutionPlan,
    LoadBalancedExecutionPlan, ProviderPricing, StandardExecutionPlan,
};
pub use metrics::{ExecutionPlanMetrics, ExecutionPlanMetricsBuilder};
pub use outcome::ExecutionOutcome;
pub use plan::{
    BoxedExecutionPlan, ExecutionPlan, ExecutionPlanBuilder as PlanBuilder, ExecutionPlanImpl,
};
pub use planner::{
    ExecutionPlanner, ExecutionPlannerBuilder, ExecutionPlannerConfig, RotationStrategyType,
};
pub use status::ExecutionPlanStatus;
pub use tracing::{DecisionLogBuilder, DecisionLogEntry, ExecutionSpan, PlanningSpan};
pub use types::{ExecutionPlanType, PlannedAccount};
