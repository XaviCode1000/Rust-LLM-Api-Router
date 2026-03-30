# Cascading Routing — Exploration Report

## Overview

Cascading Routing is a strategy where you start with the cheapest model, evaluate quality, and escalate to more capable models if the response doesn't meet quality thresholds. This report details how to implement this in the existing Rust LLM API Router architecture.

## 1. Architecture Overview of Execution Plans

### Core Trait: `ExecutionPlan`

Located at: `src/app/services/execution_plan/plan.rs`

```rust
pub trait ExecutionPlan: Send + Sync {
    fn plan_type(&self) -> ExecutionPlanType;
    fn planned_accounts(&self) -> &[PlannedAccount];
    fn status(&self) -> ExecutionPlanStatus;
    fn context(&self) -> &ExecutionContext;
    fn max_retries(&self) -> u32;
    fn timeout_seconds(&self) -> u32;
    fn outcome(&self) -> Option<ExecutionOutcome>;
    fn error_message(&self) -> Option<&str>;
    fn update_status(&mut self, status: ExecutionPlanStatus);
    fn set_outcome(&mut self, outcome: ExecutionOutcome);
    fn set_error(&mut self, message: impl Into<String>);
}
```

### Existing Plan Types

| Plan Type | Description | File Location |
|-----------|-------------|---------------|
| `Standard` | Single account execution | `implementations.rs` L40-128 |
| `Failover` | Sequential fallback on failure | `implementations.rs` L130-239 |
| `LoadBalanced` | Health-weighted distribution | `implementations.rs` L241-393 |
| `CostOptimized` | Cheapest provider selection | `implementations.rs` L395-509 |

### Execution Flow

```
LlmRouter.route_request()
  → ExecutionContext (request_id, model, preferences)
  → ExecutionPlanner.create_plan(context)
    → select_plan_type()
    → get_available_accounts()
    → apply_rotation_strategy()
    → filter_accounts()
    → build_plan()
  → LlmRouter.execute_with_fallback(plan, request)
    → forward_to_provider() for each planned_account
    → Set outcome based on result
```

## 2. How Existing Strategies Work

### StandardExecutionPlan
- Selects a single account
- Verifies health before proceeding
- No failover, simplest execution

### FailoverExecutionPlan
- Maintains pre-ordered list of accounts
- If primary fails, tries next in sequence
- Tracks `is_primary` and `is_fallback` flags on PlannedAccount

### LoadBalancedExecutionPlan
- Distributes requests based on health scores
- Has `weights: Vec<f64>` for distribution
- `select_by_weight()` uses health-based random selection

### CostOptimizedExecutionPlan
- Sorts accounts by provider pricing
- `ProviderPricing.estimate_cost()` for cost calculation
- `cheapest_provider()` method available

## 3. Where CascadingExecutionPlan Fits

### New ExecutionPlanType Variant

Add to `src/app/services/execution_plan/types.rs`:

```rust
pub enum ExecutionPlanType {
    Standard,
    Failover,
    LoadBalanced,
    CostOptimized,
    Cascading,  // NEW
}
```

### CascadingExecutionPlan Structure

```rust
pub struct CascadingExecutionPlan {
    inner: ExecutionPlanImpl,
    // NEW: Model tiers ordered by cost (cheapest first)
    model_tiers: Vec<ModelTier>,
    // NEW: Quality thresholds per tier
    quality_thresholds: QualityConfig,
}

pub struct ModelTier {
    pub model_id: String,
    pub provider_id: String,
    pub tier_price: f64,
    pub estimated_capability: QueryComplexity,
}

pub struct QualityConfig {
    pub min_score: f64,        // Minimum quality score (0-100)
    pub min_completeness: f64, // Minimum response completeness
    pub timeout_per_tier_ms: u64,
}
```

## 4. Key Interfaces/Types Involved

### Domain Entities (src/domain/entities/mod.rs)

```rust
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

pub struct Model {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub pricing: Option<ModelPricing>,
}

pub struct ModelPricing {
    pub input_cost_per_million_tokens: f64,
    pub output_cost_per_million_tokens: f64,
}
```

### Model Selector (src/domain/services/model_selector.rs)

```rust
pub trait ModelSelector: Send + Sync {
    fn select<'a>(&self, request: &ChatRequest, available_models: &'a [Model]) 
        -> SelectionResult<&'a Model>;
    fn strategy_name(&self) -> &'static str;
}

pub struct CostAwareSelector {
    classifier: QueryClassifier,
    max_cost_per_million_tokens: Option<f64>,
}
```

### Query Classifier (src/domain/services/query_complexity.rs)

```rust
pub enum QueryComplexity {
    Low = 0,    // Simple queries, cheapest models
    Medium = 1, // Conversational, mid-tier models
    High = 2,   // Complex tasks, premium models
}

pub struct QueryClassifier {
    config: ClassifierConfig,
}
```

### LLM Gateway (src/domain/traits/mod.rs)

```rust
#[async_trait]
pub trait LlmGateway: Send + Sync {
    async fn chat(&self, request: ChatRequest, api_key: &str) -> DomainResult<ChatResponse>;
    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>>;
}
```

## 5. Integration Points with LlmRouter

### Current LlmRouter Flow

`src/app/router/llm_router.rs`:

```rust
impl<R: AccountRepository + ?Sized> LlmRouter<R> {
    pub async fn route_request(&self, request: ChatRequest, preferred_providers: Vec<String>) -> Result<ChatResponse> {
        let context = self.create_execution_context(&request, preferred_providers);
        let mut plan = self.planner.create_plan(context).await?;
        self.execute_with_fallback(&mut plan, &request).await
    }
}
```

### Cascading Integration Point

The `execute_with_fallback` method would need enhancement:

```rust
async fn execute_with_cascading(&self, plan: &mut CascadingExecutionPlan, request: &ChatRequest) -> Result<ChatResponse> {
    for model_tier in plan.model_tiers() {
        // Modify request to use this tier's model
        let tier_request = request.clone();
        tier_request.model = model_tier.model_id.clone();
        
        // Execute with this tier
        match self.forward_to_provider(&model_tier.account_id, &tier_request, &model_tier.provider_id).await {
            Ok(response) => {
                // NEW: Evaluate quality
                if self.evaluate_quality(&response, &plan.quality_thresholds()).is_acceptable() {
                    plan.set_outcome(ExecutionOutcome::Success);
                    return Ok(response);
                }
                // Quality insufficient, continue to next tier
                continue;
            }
            Err(e) => {
                // Tier failed, try next
                continue;
            }
        }
    }
    // All tiers exhausted
    plan.set_outcome(ExecutionOutcome::Failure);
    Err(Error::Internal("All model tiers failed or quality insufficient".into()))
}
```

## 6. Quality Evaluation — Missing Piece

**CRITICAL**: Quality evaluation does NOT exist in the codebase. This is the biggest gap.

### Required New Trait

```rust
// src/domain/traits/quality_evaluator.rs
#[async_trait]
pub trait QualityEvaluator: Send + Sync {
    async fn evaluate(&self, request: &ChatRequest, response: &ChatResponse) -> QualityScore;
}

pub struct QualityScore {
    pub overall_score: f64,      // 0-100
    pub completeness: f64,       // 0-100
    pub coherence: f64,          // 0-100
    pub relevance: f64,          // 0-100
    pub should_escalate: bool,   // True if quality below threshold
}
```

### Quality Evaluation Strategies

1. **Length-based**: Check if response meets minimum length
2. **Keyword-based**: Check for error indicators, truncation
3. **LLM-as-Judge**: Use a cheap model to evaluate quality
4. **Statistical**: Compare against expected response patterns

## 7. Implementation Roadmap

### Phase 1: Extend PlannedAccount

Add model awareness:

```rust
pub struct PlannedAccount {
    // ... existing fields ...
    pub model_id: Option<String>,      // NEW: model to use for this tier
    pub tier_price: Option<f64>,        // NEW: cost for this tier
    pub estimated_capability: Option<QueryComplexity>, // NEW: capability level
}
```

### Phase 2: Add CascadingExecutionPlan

New file: `src/app/services/execution_plan/cascading.rs`

- Implement `ExecutionPlan` trait
- Add model tier ordering
- Add quality threshold configuration

### Phase 3: Implement QualityEvaluator

New module: `src/domain/traits/quality_evaluator.rs`

- Create trait and default implementations
- Integrate with LlmRouter

### Phase 4: Extend LlmRouter

- Add `execute_with_cascading()` method
- Modify `route_request()` to detect cascading plans
- Add quality evaluation between attempts

### Phase 5: Configuration

Extend `PlanningOptions`:

```rust
pub struct PlanningOptions {
    // ... existing fields ...
    pub enable_cascading: bool,
    pub quality_threshold: Option<f64>,
    pub max_cascading_tiers: usize,
}
```

## 8. Design Considerations & Gotchas

### Gotchas

1. **Latency**: Cascading adds latency vs single-model routing
   - Mitigation: Set per-tier timeout limits
   - Consider: Only cascade for non-streaming requests

2. **Cost Tracking**: Each tier consumes tokens
   - Need to track cumulative cost across tiers
   - Add `total_cascading_cost` to ExecutionOutcome

3. **Model Request Modification**: Request must be modified per tier
   - Original model field becomes a "desired capability"
   - Actual model used is determined by tier

4. **Quality Evaluation Overhead**: LLM-as-Judge adds latency and cost
   - Mitigation: Use lightweight heuristics first
   - Only escalate to LLM evaluation for borderline cases

5. **Streaming**: Cascading doesn't work well with streaming
   - Can't evaluate quality until stream completes
   - Recommendation: Disable cascading for streaming requests

6. **Model Availability**: Not all accounts have all models
   - Need model-aware account filtering
   - `is_model_compatible()` in ExecutionPlanner already exists

### Design Decisions

1. **Cascading vs Failover**: Different purposes
   - Failover: Same capability, different provider
   - Cascading: Different capability levels, cost-aware

2. **Quality Threshold**: Should be configurable per-request
   - Different use cases need different quality levels
   - Add to `PlanningOptions`

3. **Tier Ordering**: Three strategies possible
   - Cheapest-first (default): Start cheap, escalate
   - Best-first: Start expensive, downgrade if acceptable
   - Adaptive: Use QueryClassifier to start at predicted level

## 9. Relevant Files

| File | Role | Changes Needed |
|------|------|----------------|
| `src/app/services/execution_plan/plan.rs` | Core trait | Add model tier methods |
| `src/app/services/execution_plan/types.rs` | Plan types | Add Cascading variant |
| `src/app/services/execution_plan/implementations.rs` | Plan impls | Add CascadingExecutionPlan |
| `src/app/services/execution_plan/planner.rs` | Plan builder | Add cascading plan building |
| `src/app/services/execution_plan/context.rs` | Options | Add cascading options |
| `src/app/services/execution_plan/outcome.rs` | Outcomes | Add quality-related outcomes |
| `src/app/router/llm_router.rs` | Request routing | Add execute_with_cascading |
| `src/domain/services/model_selector.rs` | Model selection | Reuse CostAwareSelector logic |
| `src/domain/services/query_complexity.rs` | Complexity | Reuse for tier selection |
| `src/domain/traits/mod.rs` | Port traits | Add QualityEvaluator trait |
| `src/domain/entities/mod.rs` | Entities | Add quality types |
