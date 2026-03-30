# Cascading Routing Implementation Task Checklist

## 1. Types/Enums (Dependencies)
- [ ] Add `Cascading` variant to `ExecutionPlanType` enum in `src/app/services/execution_plan/types.rs`
- [ ] Add `model_id: Option<String>` field to `PlannedAccount` struct in `src/app/services/execution_plan/types.rs`
- [ ] Update `src/app/services/execution_plan/mod.rs` to export new types

## 2. Domain Traits/Structs (QualityEvaluator)
- [ ] Create `src/app/services/quality/` directory
- [ ] Create `src/app/services/quality/quality_evaluator.rs` with:
  - `QualityScore` struct { score: f64, is_acceptable: bool, checks_failed: Vec<String> }
  - `QualityConfig` struct { min_quality_score, min_response_length, max_tiers, per_tier_timeout_ms }
  - `HeuristicQualityEvaluator` implementation with 4 checks: completeness, structure, length, coherence
- [ ] Create `src/app/services/quality/mod.rs` to export quality types
- [ ] Update `src/app/services/mod.rs` to export the quality module

## 3. Implementation (CascadingExecutionPlan)
- [ ] Create `src/app/services/execution_plan/cascading.rs` with:
  - `CascadingTier` struct (PlannedAccount + model_id + tier_order)
  - `CascadingExecutionPlan` struct storing tiers: Vec<CascadingTier> + quality_config: QualityConfig
  - Implementation of `ExecutionPlan` trait that:
    - Tries tiers in order until quality threshold is met
    - Implements streaming guard (skip cascading when stream=true)
    - Tracks cumulative cost across tiers
    - Delegates to inner `ExecutionPlanImpl` for actual execution

## 4. Integration (Builder, Planner)
- [ ] Update `src/app/services/execution_plan/implementations.rs` to add `build_cascading()` method to `ExecutionPlanBuilder`
- [ ] Update `src/app/services/execution_plan/planner.rs` to add cascading auto-selection logic when:
  - budget_mode is enabled
  - not streaming

## 5. Tests
- [ ] Write unit tests for `QualityScore` and `QualityConfig` in `quality_evaluator.rs`
- [ ] Write unit tests for `HeuristicQualityEvaluator` in `quality_evaluator.rs`
- [ ] Write unit tests for `CascadingTier` and `CascadingExecutionPlan` in `cascading.rs`
- [ ] Write integration tests for `ExecutionPlanBuilder::build_cascading()` in `implementations.rs`
- [ ] Write tests for cascading auto-selection logic in `planner.rs`
- [ ] Write end-to-end tests for cascading routing functionality