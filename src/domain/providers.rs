//! Provider constants module - Known providers and selection types
//!
//! This module provides:
//! - [`ProviderId`]: Type-safe provider ID wrapper
//! - [`KnownProvider`]: Static provider configuration (34 providers)
//! - [`SelectionState`]: Interactive selection state
//! - [`ProviderSelection`]: Interactive selection result
//! - [`known_providers`]: Module with all known provider constants
//!
//! # Design Decisions (from design document)
//!
//! - Static constants for faster lookup, no I/O
//! - `&'static str` to avoid heap allocation
//! - Following api-newtype-safety, type-newum-ids patterns

use std::str::FromStr;

/// Type-safe provider ID wrapper
///
/// Use this to prevent stringly-typed errors and provide better type safety.
/// Follows: api-newtype-safety, type-newtype-ids from rust-skills
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(String);

impl ProviderId {
    /// Creates a new provider ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Creates a new provider ID from a string, validating the format.
    ///
    /// Returns `None` if the ID is invalid (empty or contains invalid characters).
    pub fn new_validated(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return None;
        }
        Some(Self(id))
    }

    /// Returns the provider ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ProviderId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("provider ID cannot be empty");
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err("provider ID must be lowercase alphanumeric with hyphens");
        }
        Ok(Self(s.to_string()))
    }
}

/// Fixed provider data (computed at compile time)
///
/// This represents a provider that users can select from the list.
/// Avoids heap allocation by using &'static str for IDs and names.
/// Follows: mem-with-capacity pattern (pre-allocated size)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownProvider {
    /// Provider unique identifier (lowercase, hyphens)
    pub id: &'static str,
    /// Human-readable provider name
    pub name: &'static str,
    /// Base URL for API requests
    pub base_url: &'static str,
}

impl KnownProvider {
    /// Returns the provider ID as a string.
    pub fn id(&self) -> &str {
        self.id
    }

    /// Returns the provider name as a string.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns the base URL as a string.
    pub fn base_url(&self) -> &str {
        self.base_url
    }
}

/// Interactive selection state
///
/// Follows: type-enum-states from rust-skills
/// Used to track the state of interactive provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionState {
    /// No selection made yet
    #[default]
    Pending,
    /// User selected a provider
    Selected,
    /// User cancelled (Ctrl+C or empty input)
    Cancelled,
    /// Selection invalid (provider not in list)
    Invalid,
}

/// Provider selection result with state and optional provider
///
/// Follows: type-enum-states patterns
/// Renamed to avoid conflict with model_selector::SelectionResult
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSelection {
    /// The state of the selection
    pub state: SelectionState,
    /// The selected provider ID, if any
    pub provider_id: Option<String>,
}

impl Default for ProviderSelection {
    fn default() -> Self {
        Self {
            state: SelectionState::Pending,
            provider_id: None,
        }
    }
}

impl ProviderSelection {
    /// Creates a new selection result.
    pub fn new(state: SelectionState, provider_id: Option<String>) -> Self {
        Self { state, provider_id }
    }

    /// Returns true if a provider was selected.
    pub fn is_selected(&self) -> bool {
        self.state == SelectionState::Selected && self.provider_id.is_some()
    }

    /// Returns true if selection was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.state == SelectionState::Cancelled
    }

    /// Returns true if selection is pending (no action taken).
    pub fn is_pending(&self) -> bool {
        self.state == SelectionState::Pending
    }

    /// Returns true if selection was invalid.
    pub fn is_invalid(&self) -> bool {
        self.state == SelectionState::Invalid
    }
}

/// Well-known LLM provider constants
///
/// List of 34 pre-configured providers with their IDs, names, and base URLs.
/// Stored in module for easy access and maintenance.
///
/// # Provider List (34 providers)
///
/// | ID | Name | Base URL |
/// |----|------|----------|
/// | openai | OpenAI | <https://api.openai.com/v1> |
/// | anthropic | Anthropic | <https://api.anthropic.com> |
/// | google-ai | Google AI | <https://generativelanguage.googleapis.com/v1> |
/// | mistral | Mistral AI | <https://api.mistral.ai/v1> |
/// | cohere | Cohere | <https://api.cohere.ai/v1> |
/// | ai21 | AI21 Labs | <https://api.ai21.com> |
/// | azure-openai | Azure OpenAI | <https://{resource}.openai.azure.com> |
/// | bedrock | AWS Bedrock | <https://bedrock-runtime.{region}.amazonaws.com> |
/// | vertex-ai | Google Vertex AI | <https://{region}-aiplatform.googleapis.com> |
/// | anthropic-vertex | Anthropic (Vertex) | via Vertex AI |
/// | openrouter | OpenRouter | <https://openrouter.ai/api/v1> |
/// | samba | SambaNova | <https://api.sambanova.ai/v1> |
/// | deepseek | DeepSeek | <https://api.deepseek.com/v1> |
/// | fireworks | Fireworks AI | <https://api.fireworks.ai/v1> |
/// | together | Together AI | <https://api.together.xyz/v1> |
/// | octane | Octane AI | <https://app.octane.ai/v1> |
/// | x-ai | xAI | <https://api.x.ai/v1> |
/// | meta-llama | Meta Llama (Cloud) | <https://api.llama.com> |
/// | perplexity | Perplexity | <https://api.perplexity.ai> |
/// | novita | Novita AI | <https://api.novita.ai/v1> |
/// | navigatr | Navigatr | <https://api.navigatr.io/v1> |
/// | hypereval | HyperEval | <https://api.hypereval.ai> |
/// | nitro | Nitro | <https://api.nitro.chat/v1> |
/// | ciasie | Ciasie | <https://api.ciasie.cn/v1> |
/// | tongyi | Alibaba Tongyi | <https://dashscope.aliyuncs.com> |
/// | baidu | Baidu ERNIE | <https://aip.baidubce.com> |
/// | tencent | Tencent Hunyuan | <https://hunyuan.tencentcloudapi.com> |
/// | minimax | MiniMax | <https://api.minimax.chat/v1> |
/// | claude-code | Claude Code | <https://claude-code.ai/v1> |
/// | lmstudio | LM Studio | <http://localhost:1234/v1> |
/// | ollama | Ollama | <http://localhost:11434/v1> |
/// | kaggle | Kaggle | <https://api.kaggle.com/v1> |
/// | cloudflare | Cloudflare Workers AI | <https://api.cloudflare.com/client/v4> |
/// | groq | Groq | <https://api.groq.com/openai/v1> |
pub mod known_providers {
    use super::KnownProvider;

    /// All known providers (34 total)
    ///
    /// Uses const for compile-time initialization.
    /// Follows: mem-with-capacity pattern
    pub const PROVIDERS: &[KnownProvider] = &[
        KnownProvider {
            id: "openai",
            name: "OpenAI",
            base_url: "https://api.openai.com/v1",
        },
        KnownProvider {
            id: "anthropic",
            name: "Anthropic",
            base_url: "https://api.anthropic.com",
        },
        KnownProvider {
            id: "google-ai",
            name: "Google AI",
            base_url: "https://generativelanguage.googleapis.com/v1",
        },
        KnownProvider {
            id: "mistral",
            name: "Mistral AI",
            base_url: "https://api.mistral.ai/v1",
        },
        KnownProvider {
            id: "cohere",
            name: "Cohere",
            base_url: "https://api.cohere.ai/v1",
        },
        KnownProvider {
            id: "ai21",
            name: "AI21 Labs",
            base_url: "https://api.ai21.com",
        },
        KnownProvider {
            id: "azure-openai",
            name: "Azure OpenAI",
            base_url: "https://{resource}.openai.azure.com",
        },
        KnownProvider {
            id: "bedrock",
            name: "AWS Bedrock",
            base_url: "https://bedrock-runtime.{region}.amazonaws.com",
        },
        KnownProvider {
            id: "vertex-ai",
            name: "Google Vertex AI",
            base_url: "https://{region}-aiplatform.googleapis.com",
        },
        KnownProvider {
            id: "anthropic-vertex",
            name: "Anthropic (Vertex)",
            base_url: "via Vertex AI",
        },
        KnownProvider {
            id: "openrouter",
            name: "OpenRouter",
            base_url: "https://openrouter.ai/api/v1",
        },
        KnownProvider {
            id: "samba",
            name: "SambaNova",
            base_url: "https://api.sambanova.ai/v1",
        },
        KnownProvider {
            id: "deepseek",
            name: "DeepSeek",
            base_url: "https://api.deepseek.com/v1",
        },
        KnownProvider {
            id: "fireworks",
            name: "Fireworks AI",
            base_url: "https://api.fireworks.ai/v1",
        },
        KnownProvider {
            id: "together",
            name: "Together AI",
            base_url: "https://api.together.xyz/v1",
        },
        KnownProvider {
            id: "octane",
            name: "Octane AI",
            base_url: "https://app.octane.ai/v1",
        },
        KnownProvider {
            id: "x-ai",
            name: "xAI",
            base_url: "https://api.x.ai/v1",
        },
        KnownProvider {
            id: "meta-llama",
            name: "Meta Llama (Cloud)",
            base_url: "https://api.llama.com",
        },
        KnownProvider {
            id: "perplexity",
            name: "Perplexity",
            base_url: "https://api.perplexity.ai",
        },
        KnownProvider {
            id: "novita",
            name: "Novita AI",
            base_url: "https://api.novita.ai/v1",
        },
        KnownProvider {
            id: "navigatr",
            name: "Navigatr",
            base_url: "https://api.navigatr.io/v1",
        },
        KnownProvider {
            id: "hypereval",
            name: "HyperEval",
            base_url: "https://api.hypereval.ai",
        },
        KnownProvider {
            id: "nitro",
            name: "Nitro",
            base_url: "https://api.nitro.chat/v1",
        },
        KnownProvider {
            id: "ciasie",
            name: "Ciasie",
            base_url: "https://api.ciasie.cn/v1",
        },
        KnownProvider {
            id: "tongyi",
            name: "Alibaba Tongyi",
            base_url: "https://dashscope.aliyuncs.com",
        },
        KnownProvider {
            id: "baidu",
            name: "Baidu ERNIE",
            base_url: "https://aip.baidubce.com",
        },
        KnownProvider {
            id: "tencent",
            name: "Tencent Hunyuan",
            base_url: "https://hunyuan.tencentcloudapi.com",
        },
        KnownProvider {
            id: "minimax",
            name: "MiniMax",
            base_url: "https://api.minimax.chat/v1",
        },
        KnownProvider {
            id: "claude-code",
            name: "Claude Code",
            base_url: "https://claude-code.ai/v1",
        },
        KnownProvider {
            id: "lmstudio",
            name: "LM Studio",
            base_url: "http://localhost:1234/v1",
        },
        KnownProvider {
            id: "ollama",
            name: "Ollama",
            base_url: "http://localhost:11434/v1",
        },
        KnownProvider {
            id: "kaggle",
            name: "Kaggle",
            base_url: "https://api.kaggle.com/v1",
        },
        KnownProvider {
            id: "cloudflare",
            name: "Cloudflare Workers AI",
            base_url: "https://api.cloudflare.com/client/v4",
        },
        KnownProvider {
            id: "groq",
            name: "Groq",
            base_url: "https://api.groq.com/openai/v1",
        },
    ];

    /// Returns all known providers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_llm_api_router::domain::known_providers;
    ///
    /// for provider in known_providers::all() {
    ///     println!("{}: {}", provider.name, provider.base_url);
    /// }
    /// ```
    pub fn all() -> &'static [KnownProvider] {
        PROVIDERS
    }

    /// Returns the number of known providers.
    pub fn count() -> usize {
        PROVIDERS.len()
    }

    /// Look up a provider by ID.
    ///
    /// Returns `None` if the provider ID is not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_llm_api_router::domain::known_providers;
    ///
    /// if let Some(provider) = known_providers::find("openai") {
    ///     println!("Found: {}", provider.name);
    /// }
    /// ```
    pub fn find(id: &str) -> Option<KnownProvider> {
        PROVIDERS.iter().find(|p| p.id == id).copied()
    }

    /// Look up a provider by ID (case-insensitive).
    ///
    /// Returns `None` if the provider ID is not found.
    pub fn find_case_insensitive(id: &str) -> Option<KnownProvider> {
        let id_lower = id.to_lowercase();
        PROVIDERS
            .iter()
            .find(|p| p.id.eq_ignore_ascii_case(&id_lower))
            .copied()
    }

    /// Returns provider IDs for interactive selection.
    ///
    /// Returns a slice of provider IDs in display order.
    pub fn ids() -> Vec<&'static str> {
        PROVIDERS.iter().map(|p| p.id).collect()
    }

    /// Returns provider names for interactive selection.
    ///
    /// Returns a slice of provider names in display order.
    pub fn names() -> Vec<&'static str> {
        PROVIDERS.iter().map(|p| p.name).collect()
    }

    /// Validates if a provider ID exists in the known list.
    pub fn is_known(id: &str) -> bool {
        find(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_count_is_34() {
        assert_eq!(known_providers::count(), 34);
    }

    #[test]
    fn test_known_providers_all_are_unique() {
        let ids: Vec<&str> = known_providers::ids();
        let unique: std::collections::HashSet<_> = ids.into_iter().collect();
        assert_eq!(unique.len(), 34, "Duplicate provider IDs found");
    }

    #[test]
    fn test_find_provider_by_id() {
        let provider = known_providers::find("openai");
        assert!(provider.is_some());
        let p = provider.unwrap();
        assert_eq!(p.name, "OpenAI");
        assert_eq!(p.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_find_provider_by_id_not_found() {
        let provider = known_providers::find("unknown-provider");
        assert!(provider.is_none());
    }

    #[test]
    fn test_find_case_insensitive() {
        let provider = known_providers::find_case_insensitive("OPENAI");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().id, "openai");
    }

    #[test]
    fn test_is_known() {
        assert!(known_providers::is_known("openai"));
        assert!(known_providers::is_known("anthropic"));
        assert!(!known_providers::is_known("unknown"));
    }

    #[test]
    fn test_selection_result_is_selected() {
        let result = ProviderSelection::new(SelectionState::Selected, Some("openai".to_string()));
        assert!(result.is_selected());
        assert!(!result.is_cancelled());
        assert!(!result.is_pending());
        assert!(!result.is_invalid());
    }

    #[test]
    fn test_selection_result_cancelled() {
        let result = ProviderSelection::new(SelectionState::Cancelled, None);
        assert!(result.is_cancelled());
        assert!(!result.is_selected());
    }

    #[test]
    fn test_provider_id_validation() {
        assert!("openai".parse::<ProviderId>().is_ok());
        assert!("groq-1".parse::<ProviderId>().is_ok());
        assert!("Invalid ID".parse::<ProviderId>().is_err());
        assert!("".parse::<ProviderId>().is_err());
    }

    #[test]
    fn test_known_providers_includes_groq() {
        let provider = known_providers::find("groq");
        assert!(provider.is_some());
        let p = provider.unwrap();
        assert_eq!(p.name, "Groq");
    }
}
