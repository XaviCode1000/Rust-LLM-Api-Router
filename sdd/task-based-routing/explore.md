# Exploration: Task-Based Routing (Issue #26)

## Goal
Extend `QueryClassifier` to classify task types (code, chat, reasoning, summarization, translation) alongside the existing `QueryComplexity` (Low/Medium/High), enabling task-aware model selection.

## Current State

### QueryClassifier (`src/domain/services/query_complexity.rs`)
- Classifies queries into `QueryComplexity` enum: `Low`, `Medium`, `High`
- Heuristic based on: character count, message count, keyword matching
- Keywords split into `high_complexity_keywords` (explain, analyze, design...) and `code_keywords` (code, function, algorithm...)
- Uses `ClassifierConfig` for configurable thresholds
- **455 lines**, **17 tests** — clean, well-tested

### CostAwareSelector (`src/domain/services/model_selector.rs`)
- Implements `ModelSelector` trait with `select()` method
- Classifies via `QueryClassifier`, then picks cheapest model whose capability tier >= complexity
- Maps price → tier: <$2 Low, <$15 Medium, >=$15 High
- **605 lines**, **20+ tests**

### ExecutionPlanType (`src/app/services/execution_plan/types.rs`)
- Current variants: `Standard`, `Failover`, `LoadBalanced`, `CostOptimized`, `Cascading`
- Each variant has `name()`, `supports_failover()`, `supports_load_balancing()`, `is_cost_optimized()`, `supports_cascading()` methods
- **235 lines**

### Routing Flow
1. HTTP → `chat_handler.rs` → `LlmRouter::route_request()`
2. `LlmRouter` creates `ExecutionContext` → `ExecutionPlanner::create_plan()`
3. `ExecutionPlanner::select_plan_type()` checks config/context options
4. Plan is built and executed with failover
5. **No task classification in the current flow** — only complexity via CostAwareSelector

### Key Domain Types (`src/domain/entities/mod.rs`)
- `ChatRequest { model, messages, temperature, max_tokens, stream }`
- `Message { role, content }`
- `Model { id, name, provider_id, pricing }`

## Affected Areas
- `src/domain/services/query_complexity.rs` — Add `TaskType` enum, extend `classify()` to return both complexity + task type
- `src/domain/services/model_selector.rs` — Potentially use task type for model family selection
- `src/domain/services/mod.rs` — Re-export new types
- `src/domain/mod.rs` — Re-export new types
- `src/app/services/execution_plan/types.rs` — Optional: new plan variant if needed
- `src/app/services/execution_plan/context.rs` — Optional: add task type to context
- New test cases for task classification

## Approaches

### 1. Extend QueryClassifier with TaskType enum (RECOMMENDED)
- Add `TaskType` enum: `Chat`, `Code`, `Reasoning`, `Summarization`, `Translation`, `General`
- Add `classify_task()` method alongside existing `classify()`
- Or return a composite `QueryClassification { complexity, task_type }`
- Add `task_keywords` to `ClassifierConfig`
- Keep backward compatibility — existing `classify()` still works

**Pros:**
- Minimal surface area change
- Follows existing patterns (keyword-based heuristics)
- Clean Architecture compliant (domain layer)
- Easy to extend with new task types
- Reuses existing ClassifierConfig pattern

**Cons:**
- Keyword-based heuristics are limited (no ML/NLP)
- Task type boundaries can be fuzzy

**Effort: Low**

### 2. New TaskClassifier service (NOT recommended)
- Separate struct/classifier in domain/services/
- Independent from QueryClassifier

**Pros:**
- Separation of concerns
- Can evolve independently

**Cons:**
- Duplicates classification logic
- More complexity for minimal benefit
- Goes against Issue #26's comment: "extend existing QueryClassifier"

**Effort: Medium**

### 3. New ExecutionPlanType::TaskOptimized variant
- Add `TaskOptimized` variant that uses task type for model selection

**Pros:**
- Clean integration with execution plan system

**Cons:**
- May be premature — task type can be used within existing plan types (CostOptimized, Cascading)

**Effort: Low (but possibly unnecessary)**

## Recommendation
**Approach 1** — Extend `QueryClassifier` with a `TaskType` enum and composite classification return type. This aligns with the Issue #26 comment and the existing architecture.

### Implementation sketch:
```rust
// In query_complexity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskType {
    #[default]
    General,
    Chat,
    Code,
    Reasoning,
    Summarization,
    Translation,
}

#[derive(Debug, Clone)]
pub struct QueryClassification {
    pub complexity: QueryComplexity,
    pub task_type: TaskType,
}

impl QueryClassifier {
    // Existing method preserved for backward compatibility
    pub fn classify(&self, request: &ChatRequest) -> QueryComplexity { ... }

    // New method returns full classification
    pub fn classify_full(&self, request: &ChatRequest) -> QueryClassification { ... }
}
```

### Model mapping (future integration):
- `TaskType::Code` → prefer Codestral, DeepSeek-Coder, GPT-4
- `TaskType::Reasoning` → prefer o3, Claude Opus, GPT-4 Turbo
- `TaskType::Chat` → prefer fast cheap models (GPT-4o-mini, Haiku)
- `TaskType::Summarization` → prefer mid-tier models
- `TaskType::Translation` → prefer models with multilingual strength

## Risks
- Keyword-based classification is inherently fuzzy — "write a function" could be Code or Chat
- Task type model mapping needs model capability data that may not exist in the current `Model` struct
- Need to decide if task type should influence `CostAwareSelector` or be a separate selector

## Ready for Proposal
Yes — the codebase is well-structured, the extension point is clear, and the scope is small.

## Relevant Files
- `src/domain/services/query_complexity.rs` — Primary file to modify (add TaskType, extend classify)
- `src/domain/services/model_selector.rs` — Secondary (may consume TaskType later)
- `src/domain/services/mod.rs` — Re-exports
- `src/domain/mod.rs` — Re-exports
- `src/app/services/execution_plan/types.rs` — ExecutionPlanType enum
- `src/app/services/execution_plan/context.rs` — ExecutionContext/PlanningOptions
- `src/app/services/execution_plan/planner.rs` — ExecutionPlanner (plan type selection)
- `src/app/router/llm_router.rs` — Request routing entry point
- `src/interfaces/handlers/chat_handler.rs` — HTTP handler
- `src/domain/entities/mod.rs` — ChatRequest, Message, Model
