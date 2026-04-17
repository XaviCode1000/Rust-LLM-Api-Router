# Technical Design: Provider List + Interactive Selection

## 1. Overview

This document specifies the implementation for adding a pre-defined list of 34 LLM providers with interactive selection support to the CLI.

### Context from Explore Phase
- CLI commands already exist: provider add/list/models/remove/enable/disable/validate
- AddProviderArgs requires: id, name, base_url, api_key (optional), interactive (optional)
- secure_storage already implemented (keyring + encrypted file)

## 2. Data Structure

### 2.1 Provider Constants

Newtype pattern following `api-newtype-safety` from rust-skills:

```rust
/// Type-safe provider ID wrapper
/// 
/// Use this to prevent stringly-typed errors and provide better type safety.
/// Follows: api-newtype-safety, type-newtype-ids
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(String);

impl ProviderId {
    pub fn as_str(&self) -> &str { &self.0 }
}

impl TryFrom<String> for ProviderId {
    type Error = DomainError;
    // Validates format: lowercase alphanumeric + hyphens
}

/// Well-known LLM provider constants
/// 
/// List of 34 pre-configured providers with their IDs, names, and base URLs.
/// Stored in module for easy access and maintenance.
pub mod known_providers {
    use super::{DomainResult, KnownProvider};
    
    /// Returns all known providers
    pub fn all() -> Vec<KnownProvider> { /* ... */ }
    
    /// Look up provider by ID
    pub fn find(id: &str) -> Option<KnownProvider> { /* ... */ }
    
    /// Returns provider IDs for interactive selection
    pub fn ids() -> Vec<&'static str> { /* ... */ }
}

/// Fixed provider data (computed at compile time)
/// 
/// This represents a provider that users can select from the list.
/// Avoids heap allocation by using &'static str for IDs and names.
#[derive(Debug, Clone, Copy)]
pub struct KnownProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
}
```

### 2.2 Enum for Selection States (following type-enum-states)

```rust
/// Interactive selection state
/// 
/// Follows: type-enum-states
/// Used to track the state of interactive provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionState {
    /// No selection made yet
    Pending,
    /// User selected a provider
    Selected,
    /// User cancelled (Ctrl+C or empty input)
    Cancelled,
    /// Selection invalid (provider not in list)
    Invalid,
}

/// Selection result with state and optional provider
#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub state: SelectionState,
    pub provider_id: Option<String>,
}
```

### 2.3 Constants for 34 Providers

| ID | Name | Base URL |
|----|------|----------|
| openai | OpenAI | https://api.openai.com/v1 |
| anthropic | Anthropic | https://api.anthropic.com |
| google-ai | Google AI | https://generativelanguage.googleapis.com/v1 |
| mistral | Mistral AI | https://api.mistral.ai/v1 |
| cohere | Cohere | https://api.cohere.ai/v1 |
| ai21 | AI21 Labs | https://api.ai21.com |
| azure-openai | Azure OpenAI | https://{resource}.openai.azure.com |
| bedrock | AWS Bedrock | https://bedrock-runtime.{region}.amazonaws.com |
| vertex-ai | Google Vertex AI | https://{region}-aiplatform.googleapis.com |
| anthropic-vertex | Anthropic (Vertex) | via Vertex AI |
| openrouter | OpenRouter | https://openrouter.ai/api/v1 |
| samba | SambaNova | https://api.sambanova.ai/v1 |
| deepseek | DeepSeek | https://api.deepseek.com/v1 |
| fireworks | Fireworks AI | https://api.fireworks.ai/v1 |
| together | Together AI | https://api.together.xyz/v1 |
| octane | Octane AI | https://app.octane.ai/v1 |
| x-ai | xAI | https://api.x.ai/v1 |
| meta-llama | Meta Llama (Cloud) | https://api.llama.com |
| perplexity | Perplexity | https://api.perplexity.ai |
| novita | Novita AI | https://api.novita.ai/v1 |
| navigatr | Navigatr | https://api.navigatr.io/v1 |
| hypereval | HyperEval | https://api.hypereval.ai |
| nitro | Nitro | https://api.nitro.chat/v1 |
| ciasie | Ciasie | https://api.ciasie.cn/v1 |
| tongyi | Alibaba Tongyi | https://dashscope.aliyuncs.com |
| baidu | Baidu ERNIE | https://aip.baidubce.com |
| tencent | Tencent Hunyuan | https://hunyuan.tencentcloudapi.com |
| minimax | MiniMax | https://api.minimax.chat/v1 |
| claude-code | Claude Code | https://claude-code.ai/v1 |
| lmstudio | LM Studio | http://localhost:1234/v1 |
| ollama | Ollama | http://localhost:11434/v1 |
| kaggle | Kaggle | https://api.kaggle.com/v1 |
| cloudflare | Cloudflare Workers AI | https://api.cloudflare.com/client/v4 |
| groq | Groq | https://api.groq.com/openai/v1 |

## 3. File Locations

### 3.1 New Files to Create

| File | Purpose |
|------|---------|
| `src/domain/providers.rs` | Provider constants (new module) |
| `src/presentation/cli/commands/provider_list.rs` | Interactive provider display |

### 3.2 Files to Modify

| File | Modification |
|------|------------|
| `src/domain/mod.rs` | Add `pub mod providers;` |
| `src/presentation/cli/commands/provider.rs` | Add `pub mod provider_list;`, modify AddProviderArgs |

## 4. CLI Integration

### 4.1 Modified AddProviderArgs

Following `err-no-unwrap-prod` (no unwrap in production) and `api-impl-into` (accept impl Into<String>):

```rust
#[derive(Debug, Args)]
pub struct AddProviderArgs {
    /// Provider ID (or use --interactive to select from list)
    #[arg(long)]
    pub id: Option<String>,

    /// Human-readable provider name (auto-filled with --interactive)
    #[arg(long)]
    pub name: Option<String>,

    /// Base URL (auto-filled with --interactive)
    #[arg(long)]
    pub base_url: Option<String>,

    /// API key for authentication
    #[arg(long)]
    pub api_key: Option<String>,

    /// Start disabled
    #[arg(long)]
    pub disabled: bool,

    /// Interactive mode: select provider from list
    #[arg(long, short)]
    pub interactive: bool,

    /// List all known providers and exit
    #[arg(long)]
    pub list: bool,
}
```

### 4.2 Modified cmd_add_provider Flow (following err-result-over-panic)

```
┌─────────────────┐
│  Start          │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ --list flag?    │──YES──► Display all providers, exit
└────────┬────────┘
         │NO
         ▼
┌─────────────────┐
│ --interactive?   │──YES──► Interactive selection flow
└────────┬────────┘
         │NO
         ▼
┌─────────────────┐
│ Check id/name/  │──NO───► Error: missing required args
│ base_url        │
└────────┬────────┘
         │YES
         ▼
┌─────────────────┐
│ Get API key     │──from arg or interactive prompt
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Create Provider │
│ and save        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Success output  │
└─────────────────┘
```

### 4.3 Interactive Selection Implementation

Following `async-no-lock-await` and proper error handling:

```rust
/// Interactive provider selection
/// 
/// Displays numbered list, accepts input (number or ID),
/// returns selected provider data or error.
/// Follows: err-no-unwrap-prod, type-enum-states
pub async fn select_provider_interactive() -> Result<SelectionResult> {
    let providers = known_providers::all();
    
    // Display numbered list
    output::info("Available providers:\n");
    for (i, p) in providers.iter().enumerate() {
        println!("  [{}] {}", i + 1, p.name);
    }
    
    // Prompt for selection
    let input = prompt::text("Select provider (number or ID, Enter to cancel)")?;
    
    // Parse selection
    let result = if input.is_empty() {
        SelectionResult { state: SelectionState::Cancelled, provider_id: None }
    } else if let Ok(num) = input.parse::<usize>() {
        // Number selection
        if num > 0 && num <= providers.len() {
            let p = &providers[num - 1];
            SelectionResult { 
                state: SelectionState::Selected, 
                provider_id: Some(p.id.to_string()) 
            }
        } else {
            SelectionResult { state: SelectionState::Invalid, provider_id: None }
        }
    } else {
        // ID selection
        if let Some(p) = known_providers::find(&input) {
            SelectionResult { 
                state: SelectionState::Selected, 
                provider_id: Some(p.id.to_string()) 
            }
        } else {
            SelectionResult { state: SelectionState::Invalid, provider_id: None }
        }
    };
    
    Ok(result)
}
```

## 5. Backward Compatibility

### 5.1 Flag Behavior Matrix

| Scenario | --id | --name | --base_url | --interactive | Behavior |
|----------|------|--------|-----------|------------|--------------|----------|
| 1 | ✓ set | ✓ set | ✓ set | false | Use provided values (existing) |
| 2 | ✓ set | ✓ set | ✓ set | true | Use provided values, prompt for API key |
| 3 | none | none | none | true | Interactive provider selection |
| 4 | ✓ set | ✓ set | ✓ set | false | Auto-fill from known list if matches |
| 5 | --list | ignored | ignored | ignored | Show list and exit |

### 5.2 Auto-fill from Known Providers

If user provides ID that matches known provider, auto-fill name and base_url:

```rust
/// Auto-fill provider details if ID matches known provider
/// 
/// Returns (name, base_url) if found in known list,
/// or original values if not found.
/// Follows: api-impl-into for flexibility
pub fn auto_fill_details(
    id: impl Into<String>,
    name: impl Into<String>,
    base_url: impl Into<String>,
) -> (String, String, bool) {
    let id = id.into();
    let name = name.into();
    let base_url = base_url.into();
    
    if let Some(kp) = known_providers::find(&id) {
        // Use known provider values, mark as auto-filled
        (kp.name.to_string(), kp.base_url.to_string(), true)
    } else {
        // Use user-provided values
        (name, base_url, false)
    }
}
```

## 6. Error Handling

### 6.1 New Error Variants (following err-thiserror-lib)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderListError {
    #[error("provider selection cancelled")]
    SelectionCancelled,
    
    #[error("invalid selection: {0}")]
    InvalidSelection(String),
    
    #[error("provider not found in known list: {0}")]
    ProviderNotFound(String),
    
    #[error("interactive selection requires TTY")]
    NotInteractive,
}
```

### 6.2 User-facing Errors

| Error | Message | Recovery |
|-------|---------|----------|
| SelectionCancelled | "Selection cancelled" | Show list again |
| InvalidSelection | "Invalid selection. Enter a number (1-34) or provider ID" | Re-prompt |
| NotInteractive | "Interactive mode requires a terminal" | Fall back to --id flag |

## 7. Acceptance Criteria

From spec mapped to implementation:

| ID | Criterion | Implementation |
|----|-----------|----------------|
| AC1 | Display list of 34 providers | `known_providers::all()` + formatted table |
| AC2 | Interactive selection via number or ID | `select_provider_interactive()` |
| AC3 | Auto-fill name/base_url from known list | `auto_fill_details()` |
| AC4 | Backward compatible with existing flags | Flag behavior matrix |
| AC5 | Handle invalid selection gracefully | Error handling with recovery |
| AC6 | Support Ctrl+C cancellation | Empty input = cancelled |
| AC7 | --list flag shows providers | Early exit in cmd_add_provider |

## 8. Implementation Task Breakdown

1. **Create src/domain/providers.rs**
   - Define ProviderId newtype
   - Define KnownProvider struct
   - Define SelectionState enum
   - Define SelectionResult struct
   - Add all 34 provider constants

2. **Create src/presentation/cli/commands/provider_list.rs**
   - Implement `display_provider_list()`
   - Implement `select_provider_interactive()`

3. **Modify src/domain/mod.rs**
   - Add `pub mod providers;`

4. **Modify src/presentation/cli/commands/provider.rs**
   - Update AddProviderArgs (make id/name/base_url optional with --interactive)
   - Update cmd_add_provider logic
   - Add --list flag handling

5. **Tests** (following test-descriptive-names)
   - test_provider_list_displays_all_34
   - test_interactive_selection_by_number
   - test_interactive_selection_by_id
   - test_interactive_selection_cancelled
   - test_interactive_selection_invalid
   - test_auto_fill_known_provider
   - test_backward_compatibility_existing_flags

## 9. Design Decisions and Trade-offs

### Decision 1: Static vs Dynamic Provider List
- **Chosen**: Static (compile-time constants)
- **Rationale**: Faster lookup, no I/O, smaller binary
- **Trade-off**: Need to rebuild when adding providers

### Decision 2: ID as String vs Newtype
- **Chosen**: Keep as String in AddProviderArgs for CLI compatibility
- **Rationale**: Avoid breaking existing CLI patterns
- **Alternative**: Newtype in domain layer only

### Decision 3: Interactive Check Method
- **Chosen**: TTY detection
- **Rationale**: Graceful fallback when not interactive
- **Trade-off**: Not all environments support TTY detection

---

**Author**: Design Phase
**Status**: Ready for Implementation
**Related Spec**: Provider List + Interactive Selection