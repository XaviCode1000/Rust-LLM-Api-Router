use tiktoken_rs::cl100k_base;

use super::model_context_limits::get_context_limit;
use crate::domain::entities::ChatRequest;
use crate::domain::errors::DomainError;

/// Validates that a ChatRequest doesn't exceed the model's context window.
pub struct TokenValidator;

impl TokenValidator {
    /// Counts tokens in a ChatRequest using cl100k_base encoding (OpenAI default).
    ///
    /// Note: This is an approximation. Different providers use different tokenizers:
    /// - OpenAI: cl100k_base (gpt-4, gpt-3.5), o200k_base (gpt-4o)
    /// - Anthropic: custom tokenizer
    /// - Groq: similar to OpenAI (Llama-based)
    ///
    /// For non-OpenAI providers, this provides a reasonable approximation.
    pub fn count_tokens(request: &ChatRequest) -> u32 {
        let bpe = cl100k_base().expect("cl100k_base encoding should be available");

        let mut total_tokens = 0;

        for message in &request.messages {
            // Each message has overhead: role + content + formatting tokens
            // Approximation: 4 tokens per message overhead (role, separators)
            total_tokens += 4;

            // Count tokens in content
            total_tokens += bpe.encode_with_special_tokens(&message.content).len() as u32;

            // Count tokens in role
            total_tokens += bpe.encode_with_special_tokens(&message.role).len() as u32;
        }

        // Request-level overhead (system prompt formatting, etc.)
        total_tokens += 2;

        total_tokens
    }

    /// Validates that the request doesn't exceed the model's context window.
    ///
    /// Returns the token count if valid, or an error if the limit is exceeded.
    /// If the model is not in the registry, validation is skipped (returns Ok).
    pub fn validate(request: &ChatRequest) -> Result<u32, DomainError> {
        let model = extract_model_name(&request.model);
        let token_count = Self::count_tokens(request);

        if let Some(limit) = get_context_limit(model) {
            // Also consider max_tokens if specified
            let max_output = request.max_tokens.unwrap_or(0);
            let total_estimated = token_count.saturating_add(max_output);

            if total_estimated > limit {
                return Err(DomainError::TokenLimitExceeded {
                    model: model.to_string(),
                    tokens: total_estimated,
                    limit,
                });
            }
        }

        Ok(token_count)
    }
}

/// Extracts the model name from a potentially prefixed format.
/// e.g., "openai:gpt-4" → "gpt-4", "groq:llama-3.1-8b-instant" → "llama-3.1-8b-instant"
fn extract_model_name(model: &str) -> &str {
    model.split(':').next_back().unwrap_or(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Message;

    #[test]
    fn test_count_tokens_simple() {
        let request = ChatRequest::new("gpt-4", vec![Message::user("Hello, how are you?")]);
        let tokens = TokenValidator::count_tokens(&request);
        assert!(tokens > 0);
        // Should be roughly: 4 (overhead) + role tokens + content tokens + 2 (request overhead)
        assert!(tokens >= 6);
    }

    #[test]
    fn test_count_tokens_conversation() {
        let request = ChatRequest::new(
            "gpt-4",
            vec![
                Message::system("You are a helpful assistant."),
                Message::user("What is Rust?"),
                Message::assistant("Rust is a systems programming language."),
                Message::user("Tell me more."),
            ],
        );
        let tokens = TokenValidator::count_tokens(&request);
        // More messages = more tokens
        assert!(tokens > 20);
    }

    #[test]
    fn test_validate_within_limit() {
        let request = ChatRequest::new("gpt-4", vec![Message::user("Hello")]);
        let result = TokenValidator::validate(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_exceeds_limit() {
        // gpt-4 has 8,192 token limit
        // Create a request that exceeds it
        let long_content = "word ".repeat(10_000);
        let request = ChatRequest::new("gpt-4", vec![Message::user(&long_content)]);
        let result = TokenValidator::validate(&request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::TokenLimitExceeded { .. }));
    }

    #[test]
    fn test_validate_unknown_model_skipped() {
        let request = ChatRequest::new("unknown-model", vec![Message::user("Hello")]);
        let result = TokenValidator::validate(&request);
        assert!(result.is_ok()); // Unknown models skip validation
    }

    #[test]
    fn test_validate_with_max_tokens() {
        let request =
            ChatRequest::new("gpt-4", vec![Message::user("Hello")]).with_max_tokens(10_000); // Exceeds gpt-4's 8,192 limit
        let result = TokenValidator::validate(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_model_name() {
        assert_eq!(extract_model_name("gpt-4"), "gpt-4");
        assert_eq!(extract_model_name("openai:gpt-4"), "gpt-4");
        assert_eq!(extract_model_name("groq:llama-3.1-8b-instant"), "llama-3.1-8b-instant");
    }
}
