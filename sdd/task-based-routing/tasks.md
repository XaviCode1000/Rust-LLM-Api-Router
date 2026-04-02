# Tasks: Task-Based Routing (Issue #26)

## Implementation Tasks

- [x] **T1: Add TaskType enum** — Add `TaskType` enum with 6 variants (General, Chat, Code, Reasoning, Summarization, Translation) and `#[default]` on General in `src/domain/services/query_complexity.rs`
- [x] **T2: Add QueryClassification struct** — Add `QueryClassification` struct with `complexity: QueryComplexity` and `task_type: TaskType` fields in `src/domain/services/query_complexity.rs`
- [x] **T3: Implement Display for TaskType** — Add `std::fmt::Display` impl for `TaskType` returning lowercase string
- [x] **T4: Extend ClassifierConfig** — Add 5 task keyword fields (code, reasoning, chat, summarization, translation) with sensible defaults to `ClassifierConfig`
- [x] **T5: Add has_any_keyword helper** — Add private helper method `has_any_keyword(&self, text: &str, keywords: &[String]) -> bool` to `QueryClassifier`
- [x] **T6: Implement classify_task()** — Add `classify_task(&self, request: &ChatRequest) -> TaskType` method with priority-based keyword matching
- [x] **T7: Implement classify_full()** — Add `classify_full(&self, request: &ChatRequest) -> QueryClassification` method combining existing `classify()` and new `classify_task()`
- [x] **T8: Update module re-exports** — Add `TaskType` and `QueryClassification` to `src/domain/services/mod.rs` re-exports
- [x] **T9: Add task classification tests** — Add 13 new tests for task classification (greeting→chat, code→code, reasoning→reasoning, summarization→summarization, translation→translation, no keywords→general, case insensitive, classify_full returns both, backward compat, custom keywords, priority, reasoning+high complexity, QueryClassification struct)
- [x] **T10: Verify all existing tests pass** — 34/34 query_complexity tests passing (17 originales + 17 nuevos)
- [x] **T11: Run clippy and fmt** — `cargo fmt --check` ✅, clippy clean en archivos modificados ✅

## Dependencies

```
T1 → T2 → T3
T4 → T5 → T6 → T7
T8 (after T1-T7)
T9 (after T6-T7)
T10 (after T9)
T11 (after T10)
```

## Notes

- All changes are in `src/domain/services/query_complexity.rs` except T8 (mod.rs)
- No changes to HTTP handlers, execution plans, or infrastructure
- Backward compatibility: `classify()` method unchanged
- New tests go in the existing `#[cfg(test)]` module in `query_complexity.rs`
