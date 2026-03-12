//! Infrastructure layer - External integrations

pub mod http_client;
pub mod logging;
pub mod metrics;
pub mod provider;

pub use http_client::HttpClient;
pub use logging::init_logging;
pub use metrics::Metrics;
