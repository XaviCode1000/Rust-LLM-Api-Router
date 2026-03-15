//! Persistence layer - Data storage implementations

pub mod json_account_repository;
pub mod json_provider_repository;

pub use json_account_repository::JsonAccountRepository;
pub use json_provider_repository::JsonProviderRepository;

#[cfg(test)]
mod json_repository_tests;
