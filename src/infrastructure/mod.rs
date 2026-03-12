//! Infrastructure layer - External integrations and persistence
//!
//! This module provides concrete implementations for:
//! - HTTP clients for external API communication
//! - Provider implementations for specific LLM services
//! - Persistence mechanisms for configuration storage
//! - Logging and metrics collection

pub mod http_client;
pub mod logging;
pub mod metrics;
pub mod persistence;
pub mod provider;

pub use http_client::HttpClient;
pub use logging::init_logging;
pub use metrics::Metrics;
pub use persistence::JsonProviderRepository;
