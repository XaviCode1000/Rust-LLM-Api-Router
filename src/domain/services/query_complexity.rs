//! Query complexity classification for cost-aware routing.
//!
//! This module analyzes incoming chat requests to estimate their complexity,
//! which drives the model selection strategy. Simpler queries can be routed
//! to cheaper models, while complex ones need more capable (expensive) models.

use crate::domain::ChatRequest;

/// Estimated complexity of a query, used for cost-aware model selection.
///
/// The ordering implements a natural cost hierarchy:
/// `Low < Medium < High`, where each level typically maps to
/// progressively more capable (and expensive) models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum QueryComplexity {
    /// Simple queries: FAQs, confirmations, short lookups.
    /// Routed to the cheapest available model.
    #[default]
    Low = 0,
    /// Regular conversational queries.
    /// Routed to mid-tier models balancing cost and quality.
    Medium = 1,
    /// Complex tasks: reasoning, code generation, long analysis.
    /// Routed to the most capable (and expensive) models.
    High = 2,
}

impl QueryComplexity {
    /// Returns `true` if this complexity level meets or exceeds the given threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: Self) -> bool {
        *self >= threshold
    }
}

impl std::fmt::Display for QueryComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryComplexity::Low => write!(f, "low"),
            QueryComplexity::Medium => write!(f, "medium"),
            QueryComplexity::High => write!(f, "high"),
        }
    }
}

/// Configuration thresholds for the complexity classifier.
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    /// Character count below which a query is considered low complexity (default: 100).
    pub low_char_threshold: usize,
    /// Character count below which a query is considered medium complexity (default: 500).
    pub medium_char_threshold: usize,
    /// Message count at or above which complexity is at least medium (default: 4).
    pub medium_message_count: usize,
    /// Message count at or above which complexity is at least high (default: 8).
    pub high_message_count: usize,
    /// Keywords that signal high-complexity reasoning tasks.
    pub high_complexity_keywords: Vec<String>,
    /// Keywords that signal code-related tasks (at least medium complexity).
    pub code_keywords: Vec<String>,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            low_char_threshold: 100,
            medium_char_threshold: 500,
            medium_message_count: 4,
            high_message_count: 8,
            high_complexity_keywords: vec![
                "explain".to_string(),
                "analyze".to_string(),
                "compare".to_string(),
                "design".to_string(),
                "architect".to_string(),
                "implement".to_string(),
                "refactor".to_string(),
                "debug".to_string(),
                "optimize".to_string(),
                "prove".to_string(),
                "derive".to_string(),
                "evaluate".to_string(),
            ],
            code_keywords: vec![
                "code".to_string(),
                "function".to_string(),
                "algorithm".to_string(),
                "class".to_string(),
                "struct".to_string(),
                "implement".to_string(),
                "refactor".to_string(),
                "debug".to_string(),
                "compile".to_string(),
                "error".to_string(),
                "test".to_string(),
            ],
        }
    }
}

/// Classifies a [`ChatRequest`] into a [`QueryComplexity`] level.
///
/// The classification heuristic considers:
/// - Total character count across all user messages
/// - Number of messages in the conversation
/// - Presence of complexity-indicating keywords
///
/// # Examples
///
/// ```rust
/// use rust_llm_api_router::domain::{services::query_complexity::*, Message, ChatRequest};
///
/// let classifier = QueryClassifier::new();
/// let request = ChatRequest::new("gpt-4", vec![Message::user("Hi")]);
/// assert_eq!(classifier.classify(&request), QueryComplexity::Low);
/// ```
#[derive(Debug, Clone)]
pub struct QueryClassifier {
    config: ClassifierConfig,
}

impl QueryClassifier {
    /// Creates a new `QueryClassifier` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ClassifierConfig::default(),
        }
    }

    /// Creates a new `QueryClassifier` with custom configuration.
    #[must_use]
    pub fn with_config(config: ClassifierConfig) -> Self {
        Self { config }
    }

    /// Classifies a chat request into a complexity level.
    #[must_use]
    pub fn classify(&self, request: &ChatRequest) -> QueryComplexity {
        let user_messages: Vec<&str> = request
            .messages
            .iter()
            .filter(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .collect();

        let total_chars: usize = user_messages.iter().map(|m| m.len()).sum();
        let message_count = request.messages.len();
        let combined_text = user_messages.join(" ").to_lowercase();

        let mut complexity = QueryComplexity::Low;

        // 1. Character-based classification
        if total_chars >= self.config.medium_char_threshold {
            complexity = QueryComplexity::High;
        } else if total_chars >= self.config.low_char_threshold {
            complexity = complexity.max(QueryComplexity::Medium);
        }

        // 2. Message-count-based classification
        if message_count >= self.config.high_message_count {
            complexity = complexity.max(QueryComplexity::High);
        } else if message_count >= self.config.medium_message_count {
            complexity = complexity.max(QueryComplexity::Medium);
        }

        // 3. Keyword-based classification
        let has_high_complexity_keyword = self
            .config
            .high_complexity_keywords
            .iter()
            .any(|kw| combined_text.contains(kw.as_str()));

        let has_code_keyword = self
            .config
            .code_keywords
            .iter()
            .any(|kw| combined_text.contains(kw.as_str()));

        if has_high_complexity_keyword {
            complexity = complexity.max(QueryComplexity::High);
        } else if has_code_keyword {
            complexity = complexity.max(QueryComplexity::Medium);
        }

        complexity
    }
}

impl Default for QueryClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Message;

    // ========================================================================
    // QueryComplexity enum tests
    // ========================================================================

    #[test]
    fn test_complexity_ordering() {
        assert!(QueryComplexity::Low < QueryComplexity::Medium);
        assert!(QueryComplexity::Medium < QueryComplexity::High);
        assert!(QueryComplexity::Low < QueryComplexity::High);
    }

    #[test]
    fn test_complexity_default_is_low() {
        assert_eq!(QueryComplexity::default(), QueryComplexity::Low);
    }

    #[test]
    fn test_complexity_meets_threshold() {
        assert!(QueryComplexity::High.meets_threshold(QueryComplexity::Low));
        assert!(QueryComplexity::High.meets_threshold(QueryComplexity::Medium));
        assert!(QueryComplexity::High.meets_threshold(QueryComplexity::High));
        assert!(QueryComplexity::Medium.meets_threshold(QueryComplexity::Low));
        assert!(!QueryComplexity::Low.meets_threshold(QueryComplexity::Medium));
        assert!(!QueryComplexity::Low.meets_threshold(QueryComplexity::High));
    }

    #[test]
    fn test_complexity_display() {
        assert_eq!(QueryComplexity::Low.to_string(), "low");
        assert_eq!(QueryComplexity::Medium.to_string(), "medium");
        assert_eq!(QueryComplexity::High.to_string(), "high");
    }

    // ========================================================================
    // QueryClassifier tests — short/low queries
    // ========================================================================

    #[test]
    fn test_classify_short_greeting_is_low() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new("gpt-4", vec![Message::user("Hi")]);

        assert_eq!(classifier.classify(&request), QueryComplexity::Low);
    }

    #[test]
    fn test_classify_simple_question_is_low() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new("gpt-4", vec![Message::user("What is 2+2?")]);

        assert_eq!(classifier.classify(&request), QueryComplexity::Low);
    }

    #[test]
    fn test_classify_empty_user_messages_is_low() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::system("You are a helpful assistant")],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::Low);
    }

    // ========================================================================
    // QueryClassifier tests — medium queries
    // ========================================================================

    #[test]
    fn test_classify_medium_length_is_medium() {
        let classifier = QueryClassifier::new();
        // ~120 chars — above low threshold (100), below medium (500)
        let content = "I have a question about my order. Can you help me track it? \
                       The order number is 12345 and I placed it last week.";
        let request = ChatRequest::new("gpt-4", vec![Message::user(content)]);

        assert_eq!(classifier.classify(&request), QueryComplexity::Medium);
    }

    #[test]
    fn test_classify_multiple_messages_is_medium() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![
                Message::user("Hi"),
                Message::assistant("Hello!"),
                Message::user("How are you?"),
                Message::assistant("I'm good!"),
                Message::user("Great"),
            ],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::Medium);
    }

    #[test]
    fn test_classify_code_keyword_is_medium() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::user("Write a test for this function")],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::Medium);
    }

    // ========================================================================
    // QueryClassifier tests — high queries
    // ========================================================================

    #[test]
    fn test_classify_long_query_is_high() {
        let classifier = QueryClassifier::new();
        let content = "a".repeat(600); // Above medium threshold
        let request = ChatRequest::new("gpt-4", vec![Message::user(&content)]);

        assert_eq!(classifier.classify(&request), QueryComplexity::High);
    }

    #[test]
    fn test_classify_explain_keyword_is_high() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::user(
                "Explain how transformers work in neural networks",
            )],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::High);
    }

    #[test]
    fn test_classify_design_keyword_is_high() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::user(
                "Design a microservices architecture for an e-commerce platform",
            )],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::High);
    }

    #[test]
    fn test_classify_many_messages_is_high() {
        let classifier = QueryClassifier::new();
        let messages: Vec<Message> = (0..9)
            .map(|i| {
                if i % 2 == 0 {
                    Message::user(format!("User message {i}"))
                } else {
                    Message::assistant(format!("Assistant message {i}"))
                }
            })
            .collect();
        let request = ChatRequest::new("gpt-4", messages);

        assert_eq!(classifier.classify(&request), QueryComplexity::High);
    }

    // ========================================================================
    // QueryClassifier tests — custom config
    // ========================================================================

    #[test]
    fn test_custom_config_adjusts_thresholds() {
        let config = ClassifierConfig {
            low_char_threshold: 50,
            medium_char_threshold: 200,
            ..Default::default()
        };
        let classifier = QueryClassifier::with_config(config);

        // 30 chars: below 50 → Low
        let short = ChatRequest::new("gpt-4", vec![Message::user("Short query here ok")]);
        assert_eq!(classifier.classify(&short), QueryComplexity::Low);

        // 80 chars: between 50 and 200 → Medium
        let medium_content =
            "This is a medium-length query that should be classified as medium complexity";
        let medium = ChatRequest::new("gpt-4", vec![Message::user(medium_content)]);
        assert_eq!(classifier.classify(&medium), QueryComplexity::Medium);
    }

    #[test]
    fn test_custom_keywords() {
        let config = ClassifierConfig {
            high_complexity_keywords: vec!["quantum".to_string()],
            ..Default::default()
        };
        let classifier = QueryClassifier::with_config(config);

        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::user("Tell me about quantum computing")],
        );

        assert_eq!(classifier.classify(&request), QueryComplexity::High);
    }

    // ========================================================================
    // Edge cases
    // ========================================================================

    #[test]
    fn test_classify_system_messages_ignored() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![
                Message::system(&"x".repeat(1000)), // Long system message
                Message::user("Hi"),                // Short user message
            ],
        );

        // Only user messages count — should be Low
        assert_eq!(classifier.classify(&request), QueryComplexity::Low);
    }

    #[test]
    fn test_classify_mixed_roles() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![
                Message::user("Hi"),
                Message::assistant("Hello!"),
                Message::user("What is the meaning of life?"),
            ],
        );

        // 2 user messages, short total → Low
        assert_eq!(classifier.classify(&request), QueryComplexity::Low);
    }

    // ========================================================================
    // Property-based invariants
    // ========================================================================

    #[test]
    fn test_classifier_is_deterministic() {
        let classifier = QueryClassifier::new();
        let request = ChatRequest::new(
            "gpt-4",
            vec![Message::user("Explain quantum mechanics in detail")],
        );

        let first = classifier.classify(&request);
        let second = classifier.classify(&request);
        assert_eq!(first, second);
    }
}
