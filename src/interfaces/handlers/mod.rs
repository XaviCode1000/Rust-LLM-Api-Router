//! HTTP request handlers

pub mod chat_handler;
pub mod health_handler;

pub use chat_handler::{chat_completions, list_models};
pub use health_handler::{health, health_detail, list_accounts};
