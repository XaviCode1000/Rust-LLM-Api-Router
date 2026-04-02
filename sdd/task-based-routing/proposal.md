# Change Proposal: Task-Based Routing (Issue #26)

## Intent

Extend the existing `QueryClassifier` to classify **task types** (code, chat, reasoning, summarization, translation, general) alongside the existing `QueryComplexity` (Low/Medium/High), enabling task-aware model selection.

## Problem

The current routing system only considers **query complexity** (how hard is the question) when selecting models. It ignores **task type** (what kind of task is being requested). This means:

- A simple code completion request gets routed the same way as a simple chat question
- Complex reasoning tasks aren't distinguished from complex summarization tasks
- No way to prefer code-specialized models (Codestral, DeepSeek-Coder) for coding tasks
- No way to prefer reasoning-specialized models (o3, Claude Opus) for reasoning tasks

The GitHub Issue #26 explicitly requests: *"Implementar enrutamiento por tarea que dirija cada peticion al LLM mas adecuado segun el tipo de tarea: codigo, chat, reasoning, etc."*

## Scope

### Included
1. Add `TaskType` enum: `General`, `Chat`, `Code`, `Reasoning`, `Summarization`, `Translation`
2. Add `QueryClassification` struct combining `complexity` + `task_type`
3. Add `classify_full()` method to `QueryClassifier` returning full classification
4. Extend `ClassifierConfig` with task-specific keyword lists
5. Add comprehensive tests for task classification (new tests, not modifying existing ones)
6. Update module re-exports (`mod.rs` files)

### NOT Included (Future Phases)
- Wiring task type into `CostAwareSelector` or creating `TaskAwareSelector`
- Adding task capability metadata to `Model` struct
- Adding `TaskOptimized` variant to `ExecutionPlanType`
- Changes to HTTP handlers or routing flow
- Integration with the execution planner

This is a **domain-layer only** change — pure business logic, no infrastructure dependencies.

## Approach

**Approach: Extend QueryClassifier** (Approach 1 from exploration)

Add task classification as a parallel heuristic alongside complexity classification, using the same keyword-based pattern already established:

1. **New `TaskType` enum** with 6 variants, `#[default]` on `General`
2. **New `QueryClassification` struct** with both `complexity` and `task_type` fields
3. **New `classify_full()` method** that returns `QueryClassification`, calling both `classify()` (existing) and new `classify_task()` internally
4. **Extended `ClassifierConfig`** with `task_keywords: HashMap<TaskType, Vec<String>>` for configurable task detection
5. **Default task keywords** for each task type:
   - `Code`: "code", "function", "algorithm", "class", "struct", "implement", "refactor", "debug", "compile", "test", "program", "script"
   - `Reasoning`: "explain", "analyze", "compare", "design", "architect", "optimize", "prove", "derive", "evaluate", "why", "how does"
   - `Chat`: "hi", "hello", "hey", "how are you", "what's up", "help me", "tell me"
   - `Summarization`: "summarize", "summary", "brief", "tl;dr", "key points", "overview"
   - `Translation`: "translate", "translation", "in spanish", "in english", "in french"
   - `General`: fallback when no task keywords match
6. **Backward compatibility**: existing `classify()` method unchanged, all 17 existing tests pass

## Impact

### Files Affected
| File | Change |
|------|--------|
| `src/domain/services/query_complexity.rs` | Add `TaskType`, `QueryClassification`, `classify_full()`, `classify_task()`, extend `ClassifierConfig` |
| `src/domain/services/mod.rs` | Re-export `TaskType`, `QueryClassification` |
| `src/domain/mod.rs` | Re-export new types if needed |
| `tests/` | New test file or inline tests for task classification |

### Risks
- **Keyword-based classification is fuzzy**: "write a function" could be Code or Chat. Mitigation: priority-based matching (stronger keywords win), with `General` as safe fallback.
- **No model-task mapping yet**: This phase only adds classification. Model selection based on task type is a future phase.
- **Config bloat**: Adding 5 keyword lists to config. Mitigation: sensible defaults, only override if needed.

## Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| **New `TaskClassifier` service** | Duplicates logic, violates Issue #26 guidance to "extend existing QueryClassifier" |
| **New `ExecutionPlanType::TaskOptimized`** | Premature — task type can enhance existing plan types without new variant |
| **ML-based classification** | Overkill for current needs, adds external dependencies, breaks domain purity |
