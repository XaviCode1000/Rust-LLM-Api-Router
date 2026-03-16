//! HTTP handlers
//!
//! This module contains the Axum HTTP request handlers that process
//! incoming API requests and return responses to clients.
//!
//! # Handlers
//!
//! - [`chat`]: Chat completions endpoint
//! - [`metrics`]: Prometheus metrics endpoint
//!
//! # Design
//!
//! Handlers are intentionally thin - they delegate to application services
//! and only handle:
//! - Request parsing and validation
//! - Error mapping to HTTP responses
//! - Response serialization

pub mod chat;
pub mod metrics;
