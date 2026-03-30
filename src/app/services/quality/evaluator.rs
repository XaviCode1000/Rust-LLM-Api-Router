use crate::app::services::execution_plan::types::PlannedAccount;
use crate::domain::entities::AccountHealth;

/// Represents the quality score of a response with detailed evaluation results.
#[derive(Debug, Clone, PartialEq)]
pub struct QualityScore {
    /// Overall quality score (0.0 to 1.0)
    pub score: f64,
    /// Whether the score meets the acceptable threshold
    pub is_acceptable: bool,
    /// List of checks that failed during evaluation
    pub checks_failed: Vec<String>,
}

impl QualityScore {
    /// Creates a new QualityScore from individual check results.
    ///
    /// # Arguments
    ///
    /// * `passed_checks` - Number of checks that passed
    /// * `total_checks` - Total number of checks performed
    /// * `failed_checks` - Names of checks that failed
    /// * `min_quality_score` - Minimum acceptable score threshold
    ///
    /// # Returns
    ///
    /// A new QualityScore instance
    pub fn new(
        passed_checks: u32,
        total_checks: u32,
        failed_checks: Vec<String>,
        min_quality_score: f64,
    ) -> Self {
        let score = if total_checks > 0 {
            passed_checks as f64 / total_checks as f64
        } else {
            0.0
        };

        Self {
            score,
            is_acceptable: score >= min_quality_score,
            checks_failed: failed_checks,
        }
    }
}

/// Configuration for quality evaluation thresholds and limits.
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Minimum quality score required for acceptability (0.0 to 1.0)
    pub min_quality_score: f64,
    /// Minimum response length in characters to be considered acceptable
    pub min_response_length: usize,
    /// Maximum number of tiers to attempt in cascading execution
    pub max_tiers: u32,
    /// Timeout per tier in milliseconds
    pub per_tier_timeout_ms: u64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            min_quality_score: 0.75,
            min_response_length: 10,
            max_tiers: 3,
            per_tier_timeout_ms: 5000,
        }
    }
}

/// Trait defining the interface for quality evaluation strategies.
#[async_trait::async_trait]
pub trait QualityGate: Send + Sync + std::fmt::Debug {
    /// Evaluates the quality of a response for a given account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account that generated the response
    /// * `response` - The response text to evaluate
    /// * `health` - Current health snapshot of the account
    ///
    /// # Returns
    ///
    /// A QualityScore representing the evaluation results
    async fn evaluate_quality(
        &self,
        account: &PlannedAccount,
        response: &str,
        health: &AccountHealth,
    ) -> QualityScore;
}

/// Implements heuristic-based quality evaluation using multiple checks.
///
/// The evaluator performs four checks:
/// 1. Completeness: Response is non-empty and not obviously truncated
/// 2. Length: Response meets minimum length threshold
/// 3. Structure: If input looked like a JSON request, response contains valid JSON-like content
/// 4. Coherence: Response doesn't contain obvious error patterns (e.g., "I cannot", "As an AI", excessive repetition)
#[derive(Debug)]
pub struct HeuristicQualityEvaluator {
    config: QualityConfig,
}

impl HeuristicQualityEvaluator {
    /// Creates a new HeuristicQualityEvaluator with default configuration.
    pub fn new() -> Self {
        Self {
            config: QualityConfig::default(),
        }
    }

    /// Creates a new HeuristicQualityEvaluator with custom configuration.
    pub fn with_config(config: QualityConfig) -> Self {
        Self { config }
    }

    /// Checks if the response appears complete (not truncated).
    ///
    /// A response is considered complete if:
    /// - It's not empty
    /// - It doesn't end with a space or open punctuation
    /// - It doesn't end mid-sentence (no trailing comma, open bracket, etc.)
    fn check_completeness(&self, response: &str) -> bool {
        if response.is_empty() {
            return false;
        }

        // Check if ends with whitespace
        if response.ends_with(|c: char| c.is_whitespace()) {
            return false;
        }

        // Check for obvious truncation patterns
        let last_char = response.chars().last().unwrap_or('\0');
        matches!(last_char, '.' | '!' | '?' | ']' | '}' | '"' | '\'')
            || !matches!(last_char, ',' | ':' | ';' | '-' | '{' | '[')
    }

    /// Checks if the response meets the minimum length requirement.
    fn check_length(&self, response: &str) -> bool {
        response.chars().count() >= self.config.min_response_length
    }

    /// Checks if the response maintains proper structure when expected.
    ///
    /// If the input appeared to be a JSON request, checks that the response
    /// contains JSON-like content (starts with { or [ and ends with matching } or ]).
    fn check_structure(&self, _input: &str, response: &str) -> bool {
        // For now, we'll implement a simple check
        // In a real implementation, we'd analyze the input to determine expected structure

        // If response looks like it should be JSON (starts with { or [) but doesn't end properly
        let trimmed = response.trim();
        if trimmed.starts_with('{') && !trimmed.ends_with('}') {
            return false;
        }
        if trimmed.starts_with('[') && !trimmed.ends_with(']') {
            return false;
        }

        true
    }

    /// Checks if the response shows signs of being coherent and not an error message.
    ///
    /// Looks for common error patterns that indicate the model refused or failed
    /// to produce a proper response.
    fn check_coherence(&self, response: &str) -> bool {
        let lower = response.to_lowercase();

        // Common error/refusal patterns
        let error_patterns = [
            "i cannot",
            "i'm unable",
            "as an ai",
            "i am not able",
            "i do not have the ability",
            "i cannot provide",
            "i cannot generate",
            "sorry, but",
            "i apologize",
            "i don't have the capability",
        ];

        // Check for error patterns
        if error_patterns.iter().any(|&pat| lower.contains(pat)) {
            return false;
        }

        // Check for excessive repetition (same word repeated 4+ times)
        let words: Vec<&str> = lower.split_whitespace().collect();
        if words.len() >= 4 {
            for i in 0..=words.len().saturating_sub(4) {
                if words[i] == words[i + 1]
                    && words[i + 1] == words[i + 2]
                    && words[i + 2] == words[i + 3]
                {
                    // Same word repeated 4 times in a row
                    return false;
                }
            }
        }

        true
    }
}

#[async_trait::async_trait]
impl QualityGate for HeuristicQualityEvaluator {
    async fn evaluate_quality(
        &self,
        _account: &PlannedAccount,
        response: &str,
        _health: &AccountHealth,
    ) -> QualityScore {
        let mut failed_checks = Vec::new();
        let mut passed_checks = 0;

        // Check 1: Completeness
        if self.check_completeness(response) {
            passed_checks += 1;
        } else {
            failed_checks.push("completeness".to_string());
        }

        // Check 2: Length
        if self.check_length(response) {
            passed_checks += 1;
        } else {
            failed_checks.push("length".to_string());
        }

        // Check 3: Structure (using empty input for now - in real implementation would pass actual input)
        if self.check_structure("", response) {
            passed_checks += 1;
        } else {
            failed_checks.push("structure".to_string());
        }

        // Check 4: Coherence
        if self.check_coherence(response) {
            passed_checks += 1;
        } else {
            failed_checks.push("coherence".to_string());
        }

        QualityScore::new(
            passed_checks,
            4, // Total checks
            failed_checks,
            self.config.min_quality_score,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::execution_plan::types::PlannedAccount;
    use crate::domain::entities::{AccountHealth, Provider};

    fn create_test_account() -> PlannedAccount {
        let provider = Provider::new("openai", "OpenAI", "https://api.openai.com");
        let health = AccountHealth::new("test-acc");
        PlannedAccount::new("test-acc", &provider, health)
    }

    #[tokio::test]
    async fn test_heuristic_quality_evaluator_new() {
        let evaluator = HeuristicQualityEvaluator::new();
        assert_eq!(evaluator.config.min_quality_score, 0.75);
        assert_eq!(evaluator.config.min_response_length, 10);
        assert_eq!(evaluator.config.max_tiers, 3);
        assert_eq!(evaluator.config.per_tier_timeout_ms, 5000);
    }

    #[tokio::test]
    async fn test_check_completeness() {
        let evaluator = HeuristicQualityEvaluator::new();

        // Empty response
        assert!(!evaluator.check_completeness(""));

        // Ends with space
        assert!(!evaluator.check_completeness("Hello "));

        // Ends with open punctuation
        assert!(!evaluator.check_completeness("Hello,"));
        assert!(!evaluator.check_completeness("Hello:"));
        assert!(!evaluator.check_completeness("Hello;"));
        assert!(!evaluator.check_completeness("Hello-"));
        assert!(!evaluator.check_completeness("Hello{"));
        assert!(!evaluator.check_completeness("Hello["));

        // Properly ended
        assert!(evaluator.check_completeness("Hello."));
        assert!(evaluator.check_completeness("Hello!"));
        assert!(evaluator.check_completeness("Hello?"));
        assert!(evaluator.check_completeness("Hello world"));
        assert!(evaluator.check_completeness("Hello world!"));
    }

    #[tokio::test]
    async fn test_check_length() {
        let evaluator = HeuristicQualityEvaluator::new();

        // Too short
        assert!(!evaluator.check_length("Hi"));
        assert!(!evaluator.check_length("Hello"));

        // Exactly minimum length
        assert!(evaluator.check_length("Hello there")); // 11 chars

        // Longer
        assert!(evaluator.check_length("This is a longer response"));
    }

    #[tokio::test]
    async fn test_check_structure() {
        let evaluator = HeuristicQualityEvaluator::new();

        // Proper JSON-like structures
        assert!(evaluator.check_structure("", "{}"));
        assert!(evaluator.check_structure("", "[]"));
        assert!(evaluator.check_structure("", "{\"key\": \"value\"}"));
        assert!(evaluator.check_structure("", "[\"item1\", \"item2\"]"));

        // Improper JSON-like structures
        assert!(!evaluator.check_structure("", "{\"key\": \"value\""));
        assert!(!evaluator.check_structure("", "[\"item1\", \"item2\""));
        assert!(!evaluator.check_structure("", "{\"key\": \"value\" ]"));

        // Non-JSON responses should pass
        assert!(evaluator.check_structure("", "This is a normal response"));
        assert!(evaluator.check_structure("", "Another normal response"));
    }

    #[tokio::test]
    async fn test_check_coherence() {
        let evaluator = HeuristicQualityEvaluator::new();

        // Good responses
        assert!(evaluator.check_coherence("This is a valid response about machine learning."));
        assert!(evaluator.check_coherence("The quick brown fox jumps over the lazy dog."));

        // Error patterns
        assert!(!evaluator.check_coherence("I cannot answer that question."));
        assert!(!evaluator.check_coherence("As an AI, I don't have that capability."));
        assert!(!evaluator.check_coherence("Sorry, but I can't help with that."));

        // Excessive repetition
        assert!(!evaluator.check_coherence("test test test test"));
        assert!(!evaluator.check_coherence("hello hello hello hello hello"));

        // Mixed content with repetition but not excessive
        assert!(evaluator.check_coherence("The test test procedure is standard"));
        // Only 2 repeats
    }

    #[tokio::test]
    async fn test_evaluate_quality_good_response() {
        let evaluator = HeuristicQualityEvaluator::new();
        let account = create_test_account();
        let health = AccountHealth::new("test-acc");
        let response = "This is a good quality response that meets all criteria.";

        let score = evaluator
            .evaluate_quality(&account, response, &health)
            .await;

        assert!(score.is_acceptable);
        assert_eq!(score.score, 1.0);
        assert!(score.checks_failed.is_empty());
    }

    #[tokio::test]
    async fn test_evaluate_quality_bad_response() {
        let evaluator = HeuristicQualityEvaluator::new();
        let account = create_test_account();
        let health = AccountHealth::new("test-acc");
        let response = "I cannot"; // Too short and contains error pattern

        let score = evaluator
            .evaluate_quality(&account, response, &health)
            .await;

        assert!(!score.is_acceptable);
        assert_eq!(score.score, 0.5); // 2 out of 4 checks passed (structure, coherence failed)
        assert!(score.checks_failed.contains(&"length".to_string()));
        assert!(score.checks_failed.contains(&"coherence".to_string()));
    }

    #[tokio::test]
    async fn test_evaluate_quality_partial_response() {
        let evaluator = HeuristicQualityEvaluator::new();
        let account = create_test_account();
        let health = AccountHealth::new("test-acc");
        let response = "This response is okay,"; // Ends with comma - incomplete

        let score = evaluator
            .evaluate_quality(&account, response, &health)
            .await;

        assert!(score.is_acceptable); // Should be acceptable with 0.75 score (3/4 checks passed)
        assert_eq!(score.score, 0.75); // 3 out of 4 checks passed (completeness failed)
        assert!(score.checks_failed.contains(&"completeness".to_string()));
        assert_eq!(score.checks_failed.len(), 1); // Only completeness failed
    }
}
