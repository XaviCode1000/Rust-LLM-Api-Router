//! Application services

pub mod account_rotation;
pub mod failover;

pub use account_rotation::{
    AccountSelector, LatencyStrategy, RotationStrategy, RoundRobinStrategy, UserAffinityStrategy,
    WeightedStrategy,
};
pub use failover::FailoverManager;

#[cfg(test)]
mod failover_tests;

#[cfg(test)]
mod account_rotation_tests;
