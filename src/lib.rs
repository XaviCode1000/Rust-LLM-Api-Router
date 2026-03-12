//! LLM API Router - Clean Architecture implementation
//!
//! A proxy/router for LLM API requests with support for multiple providers.
//!
//! # Architecture
//!
//! This crate follows Clean Architecture principles:
//!
//! - **Domain** - Core business entities and traits
//! - **Application** - Use cases and business logic
//! - **Infrastructure** - External integrations and persistence
//! - **Presentation** - HTTP API and CLI interfaces

pub mod app;
pub mod cli;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod interfaces;
pub mod presentation;

pub use error::{Error, Result};
