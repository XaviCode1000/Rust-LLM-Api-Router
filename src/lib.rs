//! LLM API Router - Clean Architecture implementation
//!
//! A proxy/router for LLM API requests with support for multiple providers.

pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod presentation;

pub use error::{Error, Result};
