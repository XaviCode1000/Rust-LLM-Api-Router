# Technical Design: Task-Based Routing (Issue #26)

## Architecture Decision

**Extend `QueryClassifier`** in the domain layer with task classification capabilities. This is a pure domain-layer change — no infrastructure dependencies, no changes to HTTP handlers, no changes to execution plans.

## Data Flow

```
ChatRequest → QueryClassifier.classify_full()
                           ├── classify() → QueryComplexity (existing)
                           └── classify_task() → TaskType (new)
                                    ↓
                         QueryClassification { complexity, task_type }
```

## Implementation Details

### 1. New Types in `query_complexity.rs`

```rust
/// The type of task a query is performing.
///
/// Used alongside `QueryComplexity` to enable task-aware model selection.
/// Each task type may prefer different model families:
/// - `Code` → Codestral, DeepSeek-Coder, GPT-4
/// - `Reasoning` → o3, Claude Opus, GPT-4 Turbo
/// - `Chat` → fast, cheap models (GPT-4o-mini, Haiku)
/// - `Summarization` → mid-tier models
/// - `Translation` → models with multilingual strength
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskType {
    /// Fallback when no specific task is detected.
    #[default]
    General,
    /// Conversational interactions, greetings, simple questions.
    Chat,
    /// Programming, debugging, code review, algorithm design.
    Code,
    /// Analysis, explanation, comparison, design decisions.
    Reasoning,
    /// Text summarization, key points extraction.
    Summarization,
    /// Language translation tasks.
    Translation,
}

/// Complete classification of a query, combining complexity and task type.
#[derive(Debug, Clone)]
pub struct QueryClassification {
    /// How complex the query is (Low/Medium/High).
    pub complexity: QueryComplexity,
    /// What type of task the query is performing.
    pub task_type: TaskType,
}
```

### 2. Extended `ClassifierConfig`

Add task keyword fields to existing config. The existing complexity fields remain unchanged.

```rust
pub struct ClassifierConfig {
    // ... existing fields unchanged ...
    
    /// Keywords that signal code-related tasks.
    pub code_keywords: Vec<String>,
    /// Keywords that signal reasoning/analysis tasks.
    pub reasoning_keywords: Vec<String>,
    /// Keywords that signal chat/conversation tasks.
    pub chat_keywords: Vec<String>,
    /// Keywords that signal summarization tasks.
    pub summarization_keywords: Vec<String>,
    /// Keywords that signal translation tasks.
    pub translation_keywords: Vec<String>,
}
```

### 3. New Methods on `QueryClassifier`

```rust
impl QueryClassifier {
    /// Classifies a chat request into a task type.
    #[must_use]
    pub fn classify_task(&self, request: &ChatRequest) -> TaskType {
        let user_messages: Vec<&str> = request
            .messages
            .iter()
            .filter(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .collect();
        
        let combined_text = user_messages.join(" ").to_lowercase();
        
        // Priority-based matching: check each task type's keywords
        // Return first match, or General as fallback
        if self.has_any_keyword(&combined_text, &self.config.code_keywords) {
            TaskType::Code
        } else if self.has_any_keyword(&combined_text, &self.config.reasoning_keywords) {
            TaskType::Reasoning
        } else if self.has_any_keyword(&combined_text, &self.config.summarization_keywords) {
            TaskType::Summarization
        } else if self.has_any_keyword(&combined_text, &self.config.translation_keywords) {
            TaskType::Translation
        } else if self.has_any_keyword(&combined_text, &self.config.chat_keywords) {
            TaskType::Chat
        } else {
            TaskType::General
        }
    }
    
    /// Returns the full classification (complexity + task type).
    #[must_use]
    pub fn classify_full(&self, request: &ChatRequest) -> QueryClassification {
        QueryClassification {
            complexity: self.classify(request),
            task_type: self.classify_task(request),
        }
    }
    
    /// Helper: check if text contains any keyword from the list.
    fn has_any_keyword(&self, text: &str, keywords: &[String]) -> bool {
        keywords.iter().any(|kw| text.contains(kw.as_str()))
    }
}
```

### 4. Keyword Priority Strategy

**Priority order** (checked first to last):
1. `Code` — strongest signal, code keywords are specific
2. `Reasoning` — analysis keywords overlap with code but reasoning is broader
3. `Summarization` — specific keywords, low false positive rate
4. `Translation` — specific keywords, low false positive rate
5. `Chat` — broad keywords, higher false positive rate
6. `General` — fallback

**Why this order?** Code and reasoning keywords are the most specific and least likely to false-positive. Chat keywords are the broadest ("help me", "tell me") so they're checked last.

### 5. Test Strategy

**New tests** (added to existing test module in `query_complexity.rs`):
- `test_classify_task_greeting_is_chat`
- `test_classify_task_code_request_is_code`
- `test_classify_task_reasoning_request_is_reasoning`
- `test_classify_task_summarization_request_is_summarization`
- `test_classify_task_translation_request_is_translation`
- `test_classify_task_no_keywords_is_general`
- `test_classify_task_case_insensitive`
- `test_classify_full_returns_both`
- `test_classify_full_backward_compat_with_classify`
- `test_custom_task_keywords`
- `test_task_keyword_priority_code_wins`

**Existing tests**: All 17 existing tests MUST pass unchanged.

### 6. Module Re-exports

`src/domain/services/mod.rs`:
```rust
pub use query_complexity::{
    ClassifierConfig, QueryClassification, QueryClassifier, QueryComplexity, TaskType,
};
```

## Files Modified

| File | Lines Changed | Description |
|------|---------------|-------------|
| `src/domain/services/query_complexity.rs` | +80-100 | Add TaskType, QueryClassification, classify_task(), classify_full(), helper, tests |
| `src/domain/services/mod.rs` | +1-2 | Re-export new types |

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Keyword overlap causes false positives | Priority-based matching, specific keywords checked first |
| Config bloat with 5 keyword lists | Sensible defaults, only override when needed |
| Breaking existing tests | `classify()` unchanged, only additive changes |
| Task type boundaries are fuzzy | `General` fallback, priority order, extensible config |
