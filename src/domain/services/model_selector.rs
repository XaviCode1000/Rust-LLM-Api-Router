//! Model selection strategies for cost-aware routing.
//!
//! This module provides the [`ModelSelector`] trait for abstracting model selection,
//! and [`CostAwareSelector`] — a strategy that routes queries to the cheapest model
//! capable of handling the estimated query complexity.
//!
//! # Architecture
//!
//! Following hexagonal architecture, the trait is a port in the domain layer.
//! Concrete strategies are domain services. Infrastructure adapters provide
//! the list of available models from external sources.

use crate::domain::{ChatRequest, Model, ModelPricing};

use super::query_complexity::{QueryClassifier, QueryComplexity};

/// Errors that can occur during model selection.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SelectionError {
    /// No model available that meets the complexity requirement.
    #[error("no model available that meets complexity threshold '{threshold}'")]
    NoModelAvailable {
        /// The minimum complexity threshold that was requested.
        threshold: QueryComplexity,
    },

    /// No models with pricing information were provided.
    #[error("no models with pricing information provided")]
    NoPricingAvailable,

    /// The available models list was empty.
    #[error("no models provided for selection")]
    EmptyModels,
}

/// Result type for model selection operations.
pub type SelectionResult<T> = Result<T, SelectionError>;

/// Trait for model selection strategies.
///
/// Implementations decide which model to use for a given request based on
/// available models, their pricing, and the request's characteristics.
///
/// # Design Notes
///
/// This trait is intentionally synchronous — model selection is a pure
/// domain operation with no I/O. The caller is responsible for fetching
/// the available models list before invoking selection.
pub trait ModelSelector: Send + Sync {
    /// Selects the best model for the given request from the available models.
    ///
    /// # Arguments
    /// * `request` - The chat request to route
    /// * `available_models` - Models available for selection
    ///
    /// # Returns
    /// The selected model, or an error if no suitable model is found.
    fn select<'a>(
        &self,
        request: &ChatRequest,
        available_models: &'a [Model],
    ) -> SelectionResult<&'a Model>;

    /// Returns a human-readable name for this strategy (for logging/metrics).
    fn strategy_name(&self) -> &'static str;
}

/// A cost-aware model selector that routes queries to the cheapest model
/// that can handle the estimated query complexity.
///
/// # Strategy
///
/// 1. Classify the incoming query's complexity ([`QueryComplexity`])
/// 2. Filter available models that have pricing information
/// 3. Group models by their effective capability tier (based on price)
/// 4. Select the cheapest model whose tier meets or exceeds the query complexity
///
/// # Configuration
///
/// - `max_cost_per_million_tokens`: Optional ceiling — models above this price
///   are excluded regardless of complexity needs.
/// - `classifier`: Custom [`QueryClassifier`] for complexity estimation.
///
/// # Examples
///
/// ```no_run
/// use rust_llm_api_router::domain::services::model_selector::CostAwareSelector;
///
/// let selector = CostAwareSelector::new();
/// // Use selector.select(&request, &models) to pick a model
/// ```
#[derive(Debug, Clone)]
pub struct CostAwareSelector {
    classifier: QueryClassifier,
    /// Optional maximum cost per 1M tokens (input + output average) in USD.
    /// Models exceeding this are excluded from selection.
    max_cost_per_million_tokens: Option<f64>,
}

impl CostAwareSelector {
    /// Creates a new `CostAwareSelector` with default classifier configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            classifier: QueryClassifier::new(),
            max_cost_per_million_tokens: None,
        }
    }

    /// Creates a new `CostAwareSelector` with a custom classifier.
    #[must_use]
    pub fn with_classifier(classifier: QueryClassifier) -> Self {
        Self {
            classifier,
            max_cost_per_million_tokens: None,
        }
    }

    /// Sets the maximum cost per 1M tokens ceiling in USD.
    ///
    /// Models with an average cost (input + output) / 2 exceeding this
    /// value will not be selected.
    #[must_use]
    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost_per_million_tokens = Some(max_cost);
        self
    }

    /// Computes a "tier price" for a model: the average of input and output costs.
    ///
    /// This single number is used to rank models by cost.
    fn tier_price(pricing: &ModelPricing) -> f64 {
        (pricing.input_cost_per_million_tokens + pricing.output_cost_per_million_tokens) / 2.0
    }

    /// Maps a tier price to an estimated capability level.
    ///
    /// Heuristic based on typical LLM pricing in 2024-2025 ($/1M tokens):
    /// - < $2.0 → Low (budget models like gpt-4o-mini, llama-3-8b)
    /// - < $15.0 → Medium (mid-tier like gpt-4o, claude-3-sonnet)
    /// - >= $15.0 → High (premium like gpt-4-turbo, claude-3-opus)
    fn capability_tier(tier_price: f64) -> QueryComplexity {
        if tier_price < 2.0 {
            QueryComplexity::Low
        } else if tier_price < 15.0 {
            QueryComplexity::Medium
        } else {
            QueryComplexity::High
        }
    }
}

impl Default for CostAwareSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelSelector for CostAwareSelector {
    fn select<'a>(
        &self,
        request: &ChatRequest,
        available_models: &'a [Model],
    ) -> SelectionResult<&'a Model> {
        if available_models.is_empty() {
            return Err(SelectionError::EmptyModels);
        }

        // 1. Classify the query
        let complexity = self.classifier.classify(request);

        // 2. Filter to models with pricing
        let priced_models: Vec<&Model> = available_models
            .iter()
            .filter(|m| m.pricing.is_some())
            .collect();

        if priced_models.is_empty() {
            return Err(SelectionError::NoPricingAvailable);
        }

        // 3. Filter by max cost ceiling (if configured)
        let eligible: Vec<&&Model> = priced_models
            .iter()
            .filter(|m| {
                if let Some(max) = self.max_cost_per_million_tokens {
                    let pricing = m.pricing.as_ref().expect("filtered to models with pricing");
                    Self::tier_price(pricing) <= max
                } else {
                    true
                }
            })
            .collect();

        if eligible.is_empty() {
            return Err(SelectionError::NoModelAvailable {
                threshold: complexity,
            });
        }

        // 4. Find the cheapest model whose capability tier meets the query complexity
        let best = eligible
            .iter()
            .filter(|m| {
                let pricing = m.pricing.as_ref().expect("filtered to models with pricing");
                let tier = Self::capability_tier(Self::tier_price(pricing));
                tier >= complexity
            })
            .min_by(|a, b| {
                let price_a =
                    Self::tier_price(a.pricing.as_ref().expect("filtered to models with pricing"));
                let price_b =
                    Self::tier_price(b.pricing.as_ref().expect("filtered to models with pricing"));
                price_a
                    .partial_cmp(&price_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        match best {
            Some(model) => Ok(model),
            // Fallback: if no model's tier is >= complexity, use the most expensive available
            // (best quality we can get within budget)
            None => {
                let fallback = eligible
                    .iter()
                    .max_by(|a, b| {
                        let price_a = Self::tier_price(
                            a.pricing.as_ref().expect("filtered to models with pricing"),
                        );
                        let price_b = Self::tier_price(
                            b.pricing.as_ref().expect("filtered to models with pricing"),
                        );
                        price_a
                            .partial_cmp(&price_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .expect("eligible is non-empty after prior check");
                Ok(fallback)
            }
        }
    }

    fn strategy_name(&self) -> &'static str {
        "cost_aware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Message, ModelPricing};

    // ========================================================================
    // Test helpers
    // ========================================================================

    fn budget_model() -> Model {
        Model::with_pricing(
            "gpt-4o-mini",
            "GPT-4o Mini",
            "openai",
            ModelPricing::new(0.15, 0.60),
        )
    }

    fn mid_tier_model() -> Model {
        Model::with_pricing("gpt-4o", "GPT-4o", "openai", ModelPricing::new(5.0, 15.0))
    }

    fn premium_model() -> Model {
        Model::with_pricing(
            "gpt-4-turbo",
            "GPT-4 Turbo",
            "openai",
            ModelPricing::new(10.0, 30.0),
        )
    }

    fn model_without_pricing() -> Model {
        Model::new("unknown-model", "Unknown Model", "unknown")
    }

    fn low_complexity_request() -> ChatRequest {
        ChatRequest::new("gpt-4", vec![Message::user("Hi")])
    }

    fn medium_complexity_request() -> ChatRequest {
        ChatRequest::new(
            "gpt-4",
            vec![Message::user(
                "I need help with my account settings. Can you walk me through \
                 how to change my password and update my email address?",
            )],
        )
    }

    fn high_complexity_request() -> ChatRequest {
        ChatRequest::new(
            "gpt-4",
            vec![Message::user(
                "Design a distributed system architecture for a real-time \
                 collaborative document editing platform with conflict resolution",
            )],
        )
    }

    // ========================================================================
    // CostAwareSelector basic tests
    // ========================================================================

    #[test]
    fn test_select_cheapest_for_low_complexity() {
        let selector = CostAwareSelector::new();
        let models = vec![budget_model(), mid_tier_model(), premium_model()];

        let result = selector.select(&low_complexity_request(), &models).unwrap();

        assert_eq!(result.id, "gpt-4o-mini");
    }

    #[test]
    fn test_select_mid_tier_for_medium_complexity() {
        let selector = CostAwareSelector::new();
        let models = vec![budget_model(), mid_tier_model(), premium_model()];

        let result = selector
            .select(&medium_complexity_request(), &models)
            .unwrap();

        // Medium complexity should pick cheapest model with tier >= Medium
        assert_eq!(result.id, "gpt-4o");
    }

    #[test]
    fn test_select_premium_for_high_complexity() {
        let selector = CostAwareSelector::new();
        let models = vec![budget_model(), mid_tier_model(), premium_model()];

        let result = selector
            .select(&high_complexity_request(), &models)
            .unwrap();

        // "Design..." keyword triggers High → cheapest High-tier model
        assert_eq!(result.id, "gpt-4-turbo");
    }

    // ========================================================================
    // Error cases
    // ========================================================================

    #[test]
    fn test_select_empty_models_returns_error() {
        let selector = CostAwareSelector::new();
        let result = selector.select(&low_complexity_request(), &[]);

        assert!(matches!(result, Err(SelectionError::EmptyModels)));
    }

    #[test]
    fn test_select_no_pricing_returns_error() {
        let selector = CostAwareSelector::new();
        let models = vec![model_without_pricing()];

        let result = selector.select(&low_complexity_request(), &models);

        assert!(matches!(result, Err(SelectionError::NoPricingAvailable)));
    }

    #[test]
    fn test_select_mixed_models_skips_unpriced() {
        let selector = CostAwareSelector::new();
        let models = vec![model_without_pricing(), budget_model(), mid_tier_model()];

        let result = selector.select(&low_complexity_request(), &models).unwrap();

        // Should skip the unpriced model and pick budget
        assert_eq!(result.id, "gpt-4o-mini");
    }

    // ========================================================================
    // Max cost ceiling
    // ========================================================================

    #[test]
    fn test_max_cost_excludes_expensive_models() {
        // Set max cost to exclude premium tier (avg > $20.0/1M)
        let selector = CostAwareSelector::new().with_max_cost(10.0);
        let models = vec![budget_model(), mid_tier_model(), premium_model()];

        // High complexity request would normally need premium, but it's excluded
        // Fallback should pick the most expensive within budget
        let result = selector
            .select(&high_complexity_request(), &models)
            .unwrap();

        // gpt-4o has avg price (5.0 + 15.0) / 2 = 10.0 which is within budget
        assert_eq!(result.id, "gpt-4o");
    }

    #[test]
    fn test_max_cost_all_excluded_returns_error() {
        let selector = CostAwareSelector::new().with_max_cost(0.01); // Impossibly low
        let models = vec![budget_model()];

        let result = selector.select(&low_complexity_request(), &models);

        assert!(matches!(
            result,
            Err(SelectionError::NoModelAvailable { .. })
        ));
    }

    // ========================================================================
    // Fallback behavior
    // ========================================================================

    #[test]
    fn test_fallback_to_best_available_when_no_tier_matches() {
        // Only budget model available, but request is high complexity
        let selector = CostAwareSelector::new();
        let models = vec![budget_model()];

        let result = selector
            .select(&high_complexity_request(), &models)
            .unwrap();

        // Should fallback to budget model (best available)
        assert_eq!(result.id, "gpt-4o-mini");
    }

    // ========================================================================
    // Strategy metadata
    // ========================================================================

    #[test]
    fn test_strategy_name() {
        let selector = CostAwareSelector::new();
        assert_eq!(selector.strategy_name(), "cost_aware");
    }

    // ========================================================================
    // ModelPricing tests
    // ========================================================================

    #[test]
    fn test_model_pricing_estimate_cost() {
        let pricing = ModelPricing::new(5.0, 15.0);

        let cost = pricing.estimate_cost(1000, 1000);
        // (1000/1M * 5.0) + (1000/1M * 15.0) = 0.005 + 0.015 = 0.02
        assert!((cost - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_pricing_estimate_cost_fractional() {
        let pricing = ModelPricing::new(3.0, 6.0);

        let cost = pricing.estimate_cost(500, 2000);
        // (500/1M * 3.0) + (2000/1M * 6.0) = 0.0015 + 0.012 = 0.0135
        assert!((cost - 0.0135).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_pricing_zero_tokens() {
        let pricing = ModelPricing::new(10.0, 20.0);

        assert!((pricing.estimate_cost(0, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_with_pricing_constructor() {
        let model = Model::with_pricing(
            "test-model",
            "Test Model",
            "test-provider",
            ModelPricing::new(1.0, 2.0),
        );

        assert!(model.pricing.is_some());
        let pricing = model.pricing.unwrap();
        assert!((pricing.input_cost_per_million_tokens - 1.0).abs() < f64::EPSILON);
        assert!((pricing.output_cost_per_million_tokens - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_new_has_no_pricing() {
        let model = Model::new("test-model", "Test Model", "test-provider");
        assert!(model.pricing.is_none());
    }

    // ========================================================================
    // Tier mapping tests
    // ========================================================================

    #[test]
    fn test_tier_price_calculation() {
        let pricing = ModelPricing::new(5.0, 15.0);
        let tier = CostAwareSelector::tier_price(&pricing);
        // (5.0 + 15.0) / 2 = 10.0
        assert!((tier - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_capability_tier_boundaries() {
        // < $2.0/1M → Low
        assert_eq!(
            CostAwareSelector::capability_tier(1.0),
            QueryComplexity::Low
        );
        // < $15.0/1M → Medium
        assert_eq!(
            CostAwareSelector::capability_tier(5.0),
            QueryComplexity::Medium
        );
        // >= $15.0/1M → High
        assert_eq!(
            CostAwareSelector::capability_tier(20.0),
            QueryComplexity::High
        );
    }

    // ========================================================================
    // Multiple providers scenario
    // ========================================================================

    #[test]
    fn test_select_across_multiple_providers() {
        let selector = CostAwareSelector::new();
        let models = vec![
            Model::with_pricing(
                "llama-3-8b",
                "Llama 3 8B",
                "groq",
                ModelPricing::new(0.05, 0.10),
            ),
            Model::with_pricing(
                "claude-3-haiku",
                "Claude 3 Haiku",
                "anthropic",
                ModelPricing::new(0.25, 1.25),
            ),
            Model::with_pricing(
                "claude-3-sonnet",
                "Claude 3 Sonnet",
                "anthropic",
                ModelPricing::new(3.0, 15.0),
            ),
            Model::with_pricing(
                "claude-3-opus",
                "Claude 3 Opus",
                "anthropic",
                ModelPricing::new(15.0, 75.0),
            ),
            Model::with_pricing(
                "gpt-4o-mini",
                "GPT-4o Mini",
                "openai",
                ModelPricing::new(0.15, 0.60),
            ),
        ];

        // Low complexity → cheapest Low-tier
        let result = selector.select(&low_complexity_request(), &models).unwrap();
        assert_eq!(result.id, "llama-3-8b");

        // Medium complexity → cheapest Medium-tier
        let result = selector
            .select(&medium_complexity_request(), &models)
            .unwrap();
        assert_eq!(result.id, "claude-3-sonnet");

        // High complexity → cheapest High-tier
        let result = selector
            .select(&high_complexity_request(), &models)
            .unwrap();
        assert_eq!(result.id, "claude-3-opus");
    }

    #[test]
    fn test_serialization_roundtrip_model_with_pricing() {
        let model = Model::with_pricing(
            "test-model",
            "Test Model",
            "provider",
            ModelPricing::new(1.0, 2.0),
        );

        let json = serde_json::to_string(&model).expect("should serialize");
        let deserialized: Model = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(model, deserialized);
    }

    #[test]
    fn test_serialization_roundtrip_model_without_pricing() {
        let model = Model::new("test-model", "Test Model", "provider");

        let json = serde_json::to_string(&model).expect("should serialize");
        let deserialized: Model = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(model, deserialized);
        // pricing field should be absent from JSON when None
        assert!(!json.contains("pricing"));
    }
}
