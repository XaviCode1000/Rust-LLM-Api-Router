//! Gateway module - LLM provider aggregation layer

pub mod llm_gateway;

pub use llm_gateway::{default_providers, LlmGatewayImpl, ProviderConfig};
