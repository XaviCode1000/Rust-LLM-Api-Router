//! HTTP request handlers

pub mod chat_handler;

pub use chat_handler::{chat_completions, list_models};