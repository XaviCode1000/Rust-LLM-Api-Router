use std::collections::HashMap;
use std::sync::LazyLock;

/// Context window limits for known models (in tokens).
/// Uses cl100k_base encoding approximation.
static CONTEXT_LIMITS: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // OpenAI models
    m.insert("gpt-4o", 128_000);
    m.insert("gpt-4o-mini", 128_000);
    m.insert("gpt-4-turbo", 128_000);
    m.insert("gpt-4", 8_192);
    m.insert("gpt-3.5-turbo", 16_385);
    m.insert("gpt-3.5-turbo-16k", 16_385);
    m.insert("gpt-3.5-turbo-instruct", 4_096);
    m.insert("o1", 200_000);
    m.insert("o1-mini", 128_000);
    m.insert("o3-mini", 200_000);

    // Anthropic models
    m.insert("claude-3-5-sonnet-20241022", 200_000);
    m.insert("claude-3-5-sonnet-latest", 200_000);
    m.insert("claude-3-5-haiku-20241022", 200_000);
    m.insert("claude-3-opus-20240229", 200_000);
    m.insert("claude-3-sonnet-20240229", 200_000);
    m.insert("claude-3-haiku-20240307", 200_000);
    m.insert("claude-2.1", 200_000);
    m.insert("claude-2.0", 100_000);

    // Groq models
    m.insert("llama-3.3-70b-versatile", 128_000);
    m.insert("llama-3.1-8b-instant", 128_000);
    m.insert("llama-3.1-70b-versatile", 128_000);
    m.insert("llama-3.2-3b-preview", 128_000);
    m.insert("mixtral-8x7b-32768", 32_768);
    m.insert("gemma2-9b-it", 8_192);

    // Mistral models
    m.insert("mistral-large-latest", 128_000);
    m.insert("mistral-small-latest", 32_000);
    m.insert("codestral-latest", 32_000);
    m.insert("open-mistral-nemo", 128_000);

    // Cohere models
    m.insert("command-r", 128_000);
    m.insert("command-r-plus", 128_000);

    // Google models
    m.insert("gemini-2.0-flash", 1_048_576);
    m.insert("gemini-2.0-flash-lite", 1_048_576);
    m.insert("gemini-1.5-pro", 2_097_152);
    m.insert("gemini-1.5-flash", 1_048_576);

    // DeepSeek models
    m.insert("deepseek-chat", 128_000);
    m.insert("deepseek-reasoner", 128_000);

    // Default fallback for unknown models
    m
});

/// Returns the context window limit for a model in tokens.
/// Returns None if the model is not in the registry.
pub fn get_context_limit(model: &str) -> Option<u32> {
    // Try exact match first
    if let Some(&limit) = CONTEXT_LIMITS.get(model) {
        return Some(limit);
    }

    // Try matching by prefix (e.g., "gpt-4-2024-04-09" → "gpt-4")
    if model.starts_with("gpt-4o") {
        return Some(128_000);
    }
    if model.starts_with("gpt-4-turbo")
        || model.starts_with("gpt-4-0125")
        || model.starts_with("gpt-4-1106")
    {
        return Some(128_000);
    }
    if model.starts_with("gpt-4") {
        return Some(8_192);
    }
    if model.starts_with("gpt-3.5-turbo-16k") || model.starts_with("gpt-3.5-turbo-0613") {
        return Some(16_385);
    }
    if model.starts_with("gpt-3.5-turbo") {
        return Some(4_096);
    }
    if model.starts_with("claude-3-5-sonnet") {
        return Some(200_000);
    }
    if model.starts_with("claude-3-5-haiku") {
        return Some(200_000);
    }
    if model.starts_with("claude-3-opus") {
        return Some(200_000);
    }
    if model.starts_with("claude-3-sonnet") {
        return Some(200_000);
    }
    if model.starts_with("claude-3-haiku") {
        return Some(200_000);
    }
    if model.starts_with("claude-2.1") {
        return Some(200_000);
    }
    if model.starts_with("claude-2") {
        return Some(100_000);
    }
    if model.starts_with("llama-3") {
        return Some(128_000);
    }
    if model.starts_with("mixtral") {
        return Some(32_768);
    }
    if model.starts_with("mistral") {
        return Some(32_000);
    }
    if model.starts_with("command-r") {
        return Some(128_000);
    }
    if model.starts_with("gemini-2.0") {
        return Some(1_048_576);
    }
    if model.starts_with("gemini-1.5") {
        return Some(2_097_152);
    }
    if model.starts_with("deepseek") {
        return Some(128_000);
    }
    if model.starts_with("gemma") {
        return Some(8_192);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert_eq!(get_context_limit("gpt-4"), Some(8_192));
        assert_eq!(get_context_limit("claude-3-haiku-20240307"), Some(200_000));
    }

    #[test]
    fn test_prefix_match() {
        assert_eq!(get_context_limit("gpt-4-2024-04-09"), Some(8_192));
        assert_eq!(get_context_limit("gpt-4-turbo-2024-04-09"), Some(128_000));
        assert_eq!(get_context_limit("gpt-3.5-turbo-0613"), Some(4_096));
        assert_eq!(get_context_limit("claude-3-sonnet-20240229"), Some(200_000));
        assert_eq!(get_context_limit("llama-3.1-70b-versatile"), Some(128_000));
    }

    #[test]
    fn test_unknown_model() {
        assert_eq!(get_context_limit("unknown-model"), None);
    }

    #[test]
    fn test_registry_has_entries() {
        assert!(CONTEXT_LIMITS.len() > 20);
    }
}
