# Specification: Task-Based Routing (Issue #26)

## Requirements

### REQ-1: TaskType Enum
The system SHALL provide a `TaskType` enum with the following variants:
- `General` (default) — fallback when no specific task is detected
- `Chat` — conversational interactions, greetings, simple questions
- `Code` — programming, debugging, code review, algorithm design
- `Reasoning` — analysis, explanation, comparison, design decisions
- `Summarization` — text summarization, key points extraction
- `Translation` — language translation tasks

### REQ-2: QueryClassification Struct
The system SHALL provide a `QueryClassification` struct containing:
- `complexity: QueryComplexity` — existing complexity level (Low/Medium/High)
- `task_type: TaskType` — detected task type

### REQ-3: classify_full() Method
`QueryClassifier` SHALL expose a `classify_full(&self, request: &ChatRequest) -> QueryClassification` method that returns both complexity and task type in a single call.

### REQ-4: classify_task() Method
`QueryClassifier` SHALL expose a `classify_task(&self, request: &ChatRequest) -> TaskType` method for cases where only task type is needed.

### REQ-5: Task Keywords Configuration
`ClassifierConfig` SHALL include configurable task-specific keyword lists:
- `code_keywords: Vec<String>` — keywords signaling code tasks
- `reasoning_keywords: Vec<String>` — keywords signaling reasoning tasks
- `chat_keywords: Vec<String>` — keywords signaling chat/conversation tasks
- `summarization_keywords: Vec<String>` — keywords signaling summarization tasks
- `translation_keywords: Vec<String>` — keywords signaling translation tasks

### REQ-6: Default Task Keywords
The default `ClassifierConfig` SHALL include sensible defaults for all task keyword lists:
- **Code**: "code", "function", "algorithm", "class", "struct", "implement", "refactor", "debug", "compile", "test", "program", "script"
- **Reasoning**: "explain", "analyze", "compare", "design", "architect", "optimize", "prove", "derive", "evaluate", "why", "how does"
- **Chat**: "hi", "hello", "hey", "how are you", "what's up", "help me", "tell me"
- **Summarization**: "summarize", "summary", "brief", "tl;dr", "key points", "overview"
- **Translation**: "translate", "translation", "in spanish", "in english", "in french"

### REQ-7: Backward Compatibility
The existing `classify()` method SHALL remain unchanged and return only `QueryComplexity`. All existing tests SHALL pass without modification.

### REQ-8: Display Implementation
`TaskType` SHALL implement `std::fmt::Display` returning lowercase string representation.

### REQ-9: Deterministic Classification
Task classification SHALL be deterministic — same input always produces same output.

### REQ-10: Domain Purity
All new types and logic SHALL reside in the domain layer with no external dependencies except `serde` for serialization.

## Scenarios

### Scenario 1: Greeting → Chat + Low
**Given** a user message "Hi, how are you?"
**When** `classify_full()` is called
**Then** `task_type` is `Chat` and `complexity` is `Low`

### Scenario 2: Code Request → Code + Medium
**Given** a user message "Write a function to sort an array"
**When** `classify_full()` is called
**Then** `task_type` is `Code` and `complexity` is `Medium`

### Scenario 3: Reasoning Request → Reasoning + High
**Given** a user message "Explain how transformers work in neural networks and compare them to RNNs"
**When** `classify_full()` is called
**Then** `task_type` is `Reasoning` and `complexity` is `High`

### Scenario 4: Summarization Request → Summarization + Medium
**Given** a user message "Summarize the key points of this article"
**When** `classify_full()` is called
**Then** `task_type` is `Summarization` and `complexity` is `Medium`

### Scenario 5: Translation Request → Translation + Medium
**Given** a user message "Translate this text to Spanish"
**When** `classify_full()` is called
**Then** `task_type` is `Translation` and `complexity` is `Medium`

### Scenario 6: No Keywords → General + Low
**Given** a user message "What is 2+2?"
**When** `classify_full()` is called
**Then** `task_type` is `General` and `complexity` is `Low`

### Scenario 7: Multiple Task Keywords → Strongest Wins
**Given** a user message "Explain how to implement this algorithm and debug the code"
**When** `classify_full()` is called
**Then** `task_type` is determined by keyword priority (reasoning and code both present)

### Scenario 8: Backward Compatibility
**Given** existing code calling `classify()`
**When** `classify()` is called
**Then** behavior is identical to before — returns only `QueryComplexity`

### Scenario 9: Custom Task Keywords
**Given** a `ClassifierConfig` with custom `code_keywords: vec!["rust".to_string()]`
**When** `classify_task()` is called with "Write a rust program"
**Then** `task_type` is `Code`

### Scenario 10: Case Insensitivity
**Given** a user message "EXPLAIN how TRANSFORMERS work"
**When** `classify_task()` is called
**Then** `task_type` is `Reasoning` (case-insensitive matching)
