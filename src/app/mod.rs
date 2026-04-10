//! Application layer - Use cases, services, and orchestration
//!
//! This module contains the application services that orchestrate the domain logic.
//! It implements the use cases of the system, following Clean Architecture principles.
//!
//! # Architecture
//!
//! The application layer provides:
//! - **Services**: Business logic orchestration
//! - **Router**: Internal LLM request routing
//! - **Health**: Health checking service
//!
//! # Services
//!
//! ## Account Rotation (`services/account_rotation.rs`)
//!
//! Implements account selection strategies:
//! - [`RoundRobinStrategy`]: Sequential rotation
//! - [`WeightedStrategy`]: Priority-based selection
//! - [`LatencyStrategy`]: Lowest latency selection
//! - [`UserAffinityStrategy`]: Same account per user
//!
//! ## Failover (`services/failover.rs`)
//!
//! Handles automatic failover with circuit breaker pattern:
//! - Tracks account health
//! - Opens circuit after consecutive failures
//! - Auto-closes after timeout
//!
//! ## Execution Planning (`services/execution_plan/`)
//!
//! Proactive execution planning module:
//! - [`ExecutionPlanner`]: Creates optimal execution plans
//! - **Plan Types**: Standard, Failover, LoadBalanced, CostOptimized
//! - **Rotation**: RoundRobin, HealthWeighted, Priority, LRU
//! - **Metrics**: Prometheus metrics for monitoring
//! - **Tracing**: OpenTelemetry integration
//!
//! ## Authentication (`services/auth`)
//!
//! Authentication service implementation (when not using simple API keys).
//!
//! # Example
//!
//! ```rust
//! use rust_llm_api_router::app::services::failover::FailoverManager;
//! use rust_llm_api_router::domain::{AccountRepository, LlmGateway};
//!
//! // Application services use domain entities and traits
//! let failover_manager = FailoverManager::new(
//!     account_repo,
//!     llm_gateway,
//!     3,  // max_retries
//! );
//! ```
//!
//! # Design Principles
//!
//! 1. **Depends only on domain**: Application services use domain traits
//! 2. **Transaction script**: Orchestrates domain entities for use cases
//! 3. **No infrastructure logic**: Delegates to infrastructure via traits
//! 4. **Testable**: Can be tested with mock implementations of domain traits

pub mod router;
pub mod services;
