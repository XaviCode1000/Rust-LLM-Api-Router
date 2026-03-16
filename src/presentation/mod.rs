//! Presentation layer - HTTP handlers, routes, and CLI
//!
//! This module handles the presentation concerns of the application:
//! - HTTP API endpoints (via Axum)
//! - Route definitions
//! - Application state management
//! - CLI commands
//!
//! # Architecture
//!
//! The presentation layer consists of:
//! - **Handlers**: HTTP request handlers (chat, health, metrics)
//! - **Routes**: Route definitions using Axum
//! - **State**: Shared application state
//! - **CLI**: Command-line interface commands
//!
//! # HTTP Endpoints
//!
//! ## Health Endpoints
//!
//! - `GET /health` - Basic health check
//! - `GET /health/detail` - Detailed health status
//!
//! ## API Endpoints
//!
//! - `POST /v1/chat/completions` - OpenAI-compatible chat API
//! - `GET /v1/models` - List available models
//!
//! ## Management Endpoints
//!
//! - `GET /accounts` - List registered accounts
//! - `GET /metrics` - Prometheus metrics
//!
//! # Application State
//!
//! The [`AppState`] struct holds shared dependencies:
//!
//! ```rust
//! pub struct AppState {
//!     pub config: Settings,           // Application configuration
//!     pub http_client: Arc<HttpClient>, // HTTP client for LLM calls
//!     pub metrics: Arc<Metrics>,       // Prometheus metrics
//!     pub account_repo: Arc<dyn AccountRepository>,  // Account storage
//!     pub provider_repo: Arc<dyn ProviderRepository>, // Provider storage
//!     // ... more fields
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use axum::{Router, routing::post};
//! use rust_llm_api_router::presentation::{routes, state::AppState};
//!
//! let app = Router::new()
//!     .merge(routes())
//!     .with_state(state);
//! ```
//!
//! # Design Principles
//!
//! 1. **Thin handlers**: Handlers delegate to application services
//! 2. **State injection**: Dependencies via Axum state extension
//! 3. **Error mapping**: Converts domain errors to HTTP responses
//! 4. **No business logic**: Presentation only, no domain logic here

pub mod cli;
pub mod routes;
pub mod state;

pub use cli::commands;
pub use routes::routes;
pub use state::AppState;
