# Technical Design: Cascading Routing

> ⚠️ **IMPLEMENTATION STATUS: EXPERIMENTAL** — The `CascadingExecutionPlan::execute()` method is a **stub** that uses simulated costs (`cost_estimate = 1000`) and never calls the real LLM gateway. It should not be enabled in production. See [#32](https://github.com/XaviCode1000/Rust-LLM-Api-Router/issues/32) for the remediation plan.

## Overview

This document describes the design for implementing cascading routing in the Rust LLM API Router. Cascading routing tries cheaper providers first and escalates to more expensive ones based on response quality evaluation.

## Module Structure

New modules to be created:
- `src/domain/services/quality_evaluation.rs` - Quality evaluation traits and implementations
- `src/app/services/execution_plan/cascading.rs` - Cascading execution plan implementation

## Design Details

### 1. Quality Evaluation

#### QualityEvaluator Trait (Domain Layer Port)

```rust
pub trait QualityEvaluator: Send + Sync {
    /// Evaluates the quality of a response
    /// 
    /// # Arguments
    /// * `response` - The response text to evaluate
    /// * `request_context` - Context about the original request
    /// 
    /// # Returns
    /// Quality score between 0.0 and 1.0
    fn evaluate(&self, response: &str, request_context: &RequestEvaluationContext) -> f64;
}
```

#### RequestEvaluationContext

```rust
#[derive(Debug, Clone)]
pub struct RequestEvaluationContext {
    pub original_prompt: String,
    pub expected_format: Option<ResponseFormat>,
    pub min_expected_length: usize,
    pub required_keywords: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ResponseFormat {
    JSON,
    Text,
    Code,
}
```

#### HeuristicQualityEvaluator Implementation

```rust
pub struct HeuristicQualityEvaluator {
    config: QualityConfig,
}

impl HeuristicQualityEvaluator {
    pub fn new(config: QualityConfig) -> Self {
        Self { config }
    }
}

impl QualityEvaluator for HeuristicQualityEvaluator {
    fn evaluate(&self, response: &str, context: &RequestEvaluationContext) -> f64 {
        if response.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;
        let mut max_score = 0.0;

        // Length check
        max_score += 1.0;
        if response.len() >= context.min_expected_length {
            score += 1.0;
        } else {
            // Partial credit for length
            score += (response.len() as f64 / context.min_expected_length as f64).min(1.0);
        }

        // Format validation
        max_score += 1.0;
        if let Some(format) = &context.expected_format {
            match format {
                ResponseFormat::JSON => {
                    if is_valid_json(response) {
                        score += 1.0;
                    }
                }
                ResponseFormat::Text => {
                    // Basic text validation - not obviously truncated
                    if !response.ends_with(',') && !response.ends_with(':') {
                        score += 1.0;
                    }
                }
                ResponseFormat::Code => {
                    // Basic code validation - has balanced brackets
                    if has_balanced_brackets(response) {
                        score += 1.0;
                    }
                }
            }
        } else {
            // No format specified, give full points
            score += 1.0;
        }

        // Keyword presence
        max_score += 1.0;
        if context.required_keywords.is_empty() {
            score += 1.0; // No keywords required
        } else {
            let mut found_keywords = 0;
            for keyword in &context.required_keywords {
                if response.to_lowercase().contains(&keyword.to_lowercase()) {
                    found_keywords += 1;
                }
            }
            score += (found_keywords as f64 / context.required_keywords.len() as f64);
        }

        (score / max_score).min(1.0).max(0.0)
    }
}

// Helper functions
fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

fn has_balanced_brackets(s: &str) -> bool {
    let mut stack = Vec::new();
    for ch in s.chars() {
        match ch {
            '(' | '[' | '{' => stack.push(ch),
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            _ => {}
        }
    }
    stack.is_empty()
}
```

#### QualityConfig

```rust
#[derive(Debug, Clone, Default)]
pub struct QualityConfig {
    /// Minimum quality score to accept a response (0.0-1.0)
    pub min_quality_score: f64,
    
    /// Minimum response length in characters
    pub min_response_length: usize,
    
    /// Maximum number of tiers to try before giving up
    pub max_tiers: u32,
    
    /// Timeout per tier in milliseconds
    pub per_tier_timeout_ms: u64,
}

impl QualityConfig {
    pub fn new(
        min_quality_score: f64,
        min_response_length: usize,
        max_tiers: u32,
        per_tier_timeout_ms: u64,
    ) -> Self {
        Self {
            min_quality_score,
            min_response_length,
            max_tiers,
            per_tier_timeout_ms,
        }
    }
}
```

### 2. Cascading Execution Plan

#### CascadingExecutionPlan Struct

```rust
#[derive(Debug)]
pub struct CascadingExecutionPlan {
    /// The underlying execution plan
    inner: ExecutionPlanImpl,
    
    /// Quality evaluator for assessing responses
    quality_evaluator: Box<dyn QualityEvaluator>,
    
    /// Configuration for cascading behavior
    config: QualityConfig,
    
    /// Current tier being executed
    current_tier: usize,
    
    /// Results from previous tiers (for debugging/metrics)
    tier_results: Vec<TierResult>,
}

#[derive(Debug, Clone)]
pub struct TierResult {
    pub tier_index: usize,
    pub account_id: String,
    pub model_id: Option<String>,
    pub quality_score: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}
```

#### Implementation

```rust
impl CascadingExecutionPlan {
    /// Creates a new CascadingExecutionPlan
    pub fn new(
        context: ExecutionContext,
        tiers: Vec<(PlannedAccount, Option<String>)>, // (account, target_model)
        quality_evaluator: Box<dyn QualityEvaluator>,
        config: QualityConfig,
    ) -> Self {
        // For cascading, we only use the first tier's account in the underlying plan
        // The cascading logic is handled in our custom execution
        let first_account = tiers.first().map(|(acc, _)| acc.clone());
        let planned_accounts = first_account.into_iter().collect();
        
        let inner = ExecutionPlanImpl::new(
            ExecutionPlanType::Cascading,
            context,
            planned_accounts,
        )
        .with_max_retries(1) // Each tier gets one attempt
        .with_timeout(30);   // Will be overridden by per-tier timeout
        
        Self {
            inner,
            quality_evaluator,
            config,
            current_tier: 0,
            tier_results: Vec::new(),
        }
    }
    
    /// Get the account for the current tier
    fn current_tier_account(&self, tiers: &[(PlannedAccount, Option<String>)]) 
        -> Option<&(PlannedAccount, Option<String>)> {
        tiers.get(self.current_tier)
    }
    
    /// Record a tier result
    fn record_tier_result(
        &mut self,
        tier_index: usize,
        account: &PlannedAccount,
        model_id: Option<String>,
        quality_score: Option<f64>,
        success: bool,
        error: Option<String>,
    ) {
        self.tier_results.push(TierResult {
            tier_index,
            account_id: account.account_id.clone(),
            model_id,
            quality_score,
            success,
            error,
        });
    }
}

impl ExecutionPlan for CascadingExecutionPlan {
    // Delegate all standard methods to the inner plan
    fn plan_type(&self) -> ExecutionPlanType {
        self.inner.plan_type()
    }
    
    fn planned_accounts(&self) -> &[PlannedAccount] {
        self.inner.planned_accounts()
    }
    
    fn status(&self) -> ExecutionPlanStatus {
        self.inner.status()
    }
    
    fn context(&self) -> &ExecutionContext {
        self.inner.context()
    }
    
    fn max_retries(&self) -> u32 {
        self.inner.max_retries()
    }
    
    fn timeout_seconds(&self) -> u32 {
        self.inner.timeout_seconds()
    }
    
    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.inner.outcome()
    }
    
    fn error_message(&self) -> Option<&str> {
        self.inner.error_message()
    }
    
    fn update_status(&mut self, status: ExecutionPlanStatus) {
        self.inner.update_status(status);
    }
    
    fn set_outcome(&mut self, outcome: ExecutionOutcome) {
        self.inner.set_outcome(outcome);
    }
    
    fn set_error(&mut self, message: impl Into<String>) {
        self.inner.set_error(message);
    }
}
```

Note: The actual execution logic (iterating through tiers, evaluating quality, etc.) will be handled in the execution plan builder or a separate executor service, as the ExecutionPlan trait focuses on plan metadata rather than execution logic.

### 3. Integration with ExecutionPlanBuilder

#### Extended ExecutionPlanType

Already implemented in types.rs:
- Added `Cascading` variant
- Updated `supports_cascading()` method
- Updated `is_cost_optimized()` to include Cascading

#### Builder Integration

Add to `ExecutionPlanBuilder` in implementations.rs:

```rust
impl<R: AccountRepository> ExecutionPlanBuilder<R> {
    // ... existing methods ...
    
    /// Builds a CascadingExecutionPlan.
    pub async fn build_cascading(
        &self,
        context: ExecutionContext,
        quality_config: Option<QualityConfig>,
    ) -> Result<CascadingExecutionPlan, DomainError> {
        let accounts = self.get_accounts_for_context(&context).await?;
        
        // Create tiers - each account with its preferred model
        let tiers: Vec<(PlannedAccount, Option<String>)> = accounts
            .into_iter()
            .enumerate()
            .map(|(idx, (account, provider, health))| {
                // Determine target model for this account
                let target_model = context.provider_model_preferences.iter()
                    .find(|pref| pref.starts_with(&format!("{}:", provider.id)))
                    .and_then(|pref| pref.split(':').nth(1))
                    .or_else(|| Some(&context.model))
                    .cloned();
                
                let mut planned = PlannedAccount::new(account.id.clone(), &provider, health)
                    .with_execution_order(idx as u32)
                    .with_priority(account.priority)
                    .with_model_id(target_model.clone());
                
                // First account is primary, rest are fallbacks
                if idx == 0 {
                    planned = planned.as_primary();
                } else {
                    planned = planned.as_fallback();
                }
                
                (planned, target_model)
            })
            .collect();
        
        // Use default quality config if none provided
        let config = quality_config.unwrap_or_else(QualityConfig::default);
        
        // Create heuristic quality evaluator
        let quality_evaluator = Box::new(HeuristicQualityEvaluator::new(config.clone()));
        
        Ok(CascadingExecutionPlan::new(
            context,
            tiers,
            quality_evaluator,
            config,
        ))
    }
}
```

### 4. Tier Representation and Iteration

Each tier consists of:
- A `PlannedAccount` (with account/provider/health info)
- An optional `target_model` override (if different from context.model)

The cascading logic will iterate through tiers in order:
1. Execute request using current tier's account and model
2. Evaluate response quality using the QualityEvaluator
3. If quality meets threshold OR we've reached max tiers, return result
4. Otherwise, move to next tier and repeat

### 5. Integration Points

#### PlanningOptions Extension

Add cascading option to PlanningOptions:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanningOptions {
    // ... existing fields ...
    
    /// Enable cascading routing (try cheaper providers first, escalate on quality failure)
    pub enable_cascading: bool,
}

impl Default for PlanningOptions {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            enable_cascading: true, // Enable by default
        }
    }
}
```

#### Request Flow Modification

In the LLMRouter or execution service, before executing a plan:
1. Check if `planning_options.enable_cascading` is true
2. Check if request is NOT streaming (cascading incompatible with streaming)
3. If both conditions met, use `build_cascading` instead of standard builders
4. Otherwise, use existing logic

### 6. Test Strategy

#### Unit Tests
- Test QualityEvaluator implementations with various responses
- Test CascadingExecutionPlan creation and delegation methods
- Test tier progression logic
- Test config defaults and customization

#### Integration Tests
- Test end-to-end cascading flow with mock LLM gateway
- Test quality-based escalation triggers
- Test timeout handling per tier
- Test streaming guard (cascading disabled for streams)

### 7. File Layout

```
src/
├── domain/
│   ├── services/
│   │   ├── quality_evaluation.rs      # QualityEvaluator trait + implementations
│   │   └── ... existing files ...
│   └── ...
├── app/
│   ├── services/
│   │   ├── execution_plan/
│   │   │   ├── cascading.rs           # CascadingExecutionPlan implementation
│   │   │   ├── implementations.rs     # Updated with build_cascading method
│   │   │   ├── types.rs               # Already updated with Cascading variant
│   │   │   ├── context.rs             # Updated PlanningOptions with enable_cascading
│   │   │   └── ... existing files ...
│   │   └── ... existing files ...
│   └── ...
└── ...
```

## Open Questions & Decisions

### 1. Where should the actual cascading execution logic reside?
**Decision**: Create a dedicated `CascadingExecutor` service that takes a `CascadingExecutionPlan` and executes the tier-by-tier logic with quality evaluation. This keeps the ExecutionPlan trait focused on metadata.

### 2. How should quality evaluation context be populated?
**Decision**: Extract from ExecutionContext.request_params and context fields. Provide sensible defaults.

### 3. Should we track metrics for cascading decisions?
**Decision**: Yes, add metrics collection for:
- Number of tiers tried
- Quality scores per tier
- Time spent per tier
- Final selected tier

### 4. How should we handle partial success outcomes?
**Decision**: Use existing `ExecutionOutcome::PartialSuccess` when we get a response but it doesn't meet quality thresholds.

## Summary

This design introduces cascading routing by:
1. Adding a QualityEvaluator trait and HeuristicQualityEvaluator implementation
2. Creating a CascadingExecutionPlan that follows the existing delegation pattern
3. Extending ExecutionPlanBuilder with a build_cascading method
4. Adding cascading support to ExecutionPlanType and PlanningOptions
5. Representing tiers as (PlannedAccount, target_model) pairs
6. Providing comprehensive test strategy

The implementation follows existing patterns in the codebase and maintains compatibility with the hexagonal architecture.