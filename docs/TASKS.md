# Cascading Routing Implementation Task Checklist

## 1. Types/Enums (Dependencies)
- [x] Add `Cascading` variant to `ExecutionPlanType` enum in `src/app/services/execution_plan/types.rs`
- [x] Add `model_id: Option<String>` field to `PlannedAccount` struct in `src/app/services/execution_plan/types.rs`
- [x] Update `src/app/services/execution_plan/mod.rs` to export new types

## 2. Domain Traits/Structs (QualityEvaluator)
- [x] Create `src/app/services/quality/` directory
- [x] Create `src/app/services/quality/evaluator.rs` with:
  - `QualityScore` struct { score: f64, is_acceptable: bool, checks_failed: Vec<String> }
  - `QualityConfig` struct { min_quality_score, min_response_length, max_tiers, per_tier_timeout_ms }
  - `HeuristicQualityEvaluator` implementation with 4 checks: completeness, structure, length, coherence
  - `QualityGate` trait for extensible evaluation
- [x] Create `src/app/services/quality/mod.rs` to export quality types
- [x] Update `src/app/services/mod.rs` to export the quality module

## 3. Implementation (CascadingExecutionPlan)
- [x] Create `src/app/services/execution_plan/cascading.rs` with:
  - `CascadingTier` struct (PlannedAccount + model_id + tier_order)
  - `CascadingExecutionPlan` struct storing tiers: Vec<CascadingTier> + quality_config: QualityConfig
  - Implementation of `ExecutionPlan` trait that:
    - Tries tiers in order until quality threshold is met
    - Implements streaming guard (skip cascading when stream=true)
    - Tracks cumulative cost across tiers
    - Delegates to inner `ExecutionPlanImpl` for actual execution

## 4. Integration (Builder, Planner)
- [x] Update `src/app/services/execution_plan/implementations.rs` to add `build_cascading()` method to `ExecutionPlanBuilder`
- [x] Update `src/app/services/execution_plan/planner.rs` to add cascading auto-selection logic when:
  - budget_mode is enabled
  - not streaming

## 5. Tests
- [x] Write unit tests for `QualityScore` and `QualityConfig` in `evaluator.rs`
- [x] Write unit tests for `HeuristicQualityEvaluator` in `evaluator.rs`
- [x] Write unit tests for `CascadingTier` and `CascadingExecutionPlan` in `cascading.rs`
- [x] Write integration tests for cascading functionality
- [x] Tests pass (492 total tests passing)

## 6. Documentation (Issue #23 and #24)
- [x] Update README.md with Cost-Aware Routing section
- [x] Update README.md with Cascading Routing section
- [x] Update README.md with comparison table
- [x] Update README.md features list
- [x] Update README.md roadmap
- [x] Update README.md project structure
- [x] Create CHANGELOG.md with entries for both issues
- [x] Update docs/architecture.md with domain services section
- [x] Update docs/architecture.md with cascading execution plan section
- [x] Update docs/architecture.md with quality gate section
- [x] Create docs/routing.md with comprehensive routing documentation
- [x] Update docs/api.md with intelligent routing note
- [x] Update src/app/services/execution_plan/README.md with cascading section
- [x] Update TASKS.md to mark completed tasks