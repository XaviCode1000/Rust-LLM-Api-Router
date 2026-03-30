pub use self::evaluator::{HeuristicQualityEvaluator, QualityConfig, QualityGate, QualityScore};

/// Quality evaluation module for cascading execution plans.
/// Provides quality scoring and evaluation strategies to determine when to escalate
/// to a higher quality/cost tier in cascading execution.
pub mod evaluator;