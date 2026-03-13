//! Gateway module - LLM provider aggregation layer

pub mod llm_gateway;

pub use llm_gateway::{LlmGatewayImpl, ProviderConfig, default_providers};
