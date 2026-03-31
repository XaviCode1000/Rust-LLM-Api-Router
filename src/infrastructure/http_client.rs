//! HTTP client for making requests to LLM providers

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use reqwest::Client;
use std::sync::Arc;

pub struct HttpClient {
    client: Client,
    /// Optional mock URL for testing - when set, all requests go to this base URL
    mock_base_url: Option<String>,
}

impl HttpClient {
    pub fn new() -> Result<Self, crate::Error> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .default_headers(default_headers)
            .build()?;
        Ok(Self {
            client,
            mock_base_url: None,
        })
    }

    /// Create HTTP client with mock URL for testing
    pub fn with_mock_url(mock_url: &str) -> Result<Self, crate::Error> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .default_headers(default_headers)
            .build()?;
        Ok(Self {
            client,
            mock_base_url: Some(mock_url.to_string()),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the mock base URL if configured
    pub fn mock_base_url(&self) -> Option<&str> {
        self.mock_base_url.as_deref()
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

pub type SharedHttpClient = Arc<HttpClient>;
