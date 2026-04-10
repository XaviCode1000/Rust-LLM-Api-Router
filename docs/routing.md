# Routing Strategies

The LLM API Router provides multiple intelligent routing strategies for cost optimization:

- **Cost-Aware Routing** (Issue #23): Static model selection by query complexity
- **Cascading Routing** (Issue #24): Dynamic quality-based escalation
- **Task-Based Routing** (Issue #26): Route by task type (General, Chat, Code, Reasoning, Summarization, Translation)
- **Routing Configuration** (Issue #29): CLI flags and environment variables for all strategies

## Overview

Both strategies aim to reduce costs while maintaining response quality, but they operate at different stages:

```
Request Flow:
┌─────────────┐    ┌──────────────────┐    ┌─────────────┐
│   Incoming   │───▶│  Cost-Aware      │───▶│   Execute   │
│   Request    │    │  (Pre-request)   │    │   Model     │
└─────────────┘    └──────────────────┘    └─────────────┘
                          │                        │
                          │                        ▼
                   Selects cheapest         ┌─────────────┐
                   capable model            │   Evaluate   │
                                           │   Quality    │
                                           └─────────────┘
                                                  │
                                           If quality < 0.75
                                                  │
                                                  ▼
                                           ┌─────────────┐
                                           │   Escalate   │
                                           │  to next tier│
                                           └─────────────┘
```

## Cost-Aware Routing (Issue #23)

### Purpose

Routes queries to the cheapest model capable of handling the estimated complexity **before** making the request.

### Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `CostAwareSelector` | `src/domain/services/model_selector.rs` | Selects cheapest model meeting complexity |
| `QueryClassifier` | `src/domain/services/query_complexity.rs` | Classifies query complexity |
| `ModelSelector` trait | `src/domain/services/model_selector.rs` | Pluggable selection strategy |

### How It Works

1. **Classify Query**: Analyzes message length, conversation history, and keywords
2. **Map to Complexity**: Low → budget models, Medium → mid-tier, High → premium
3. **Select Model**: Picks the cheapest model whose tier meets complexity
4. **Apply Constraints**: Respects optional cost ceiling per million tokens

### Complexity Classification

```rust
pub enum QueryComplexity {
    Low = 0,    // Short queries, simple questions
    Medium = 1, // Conversational, code keywords  
    High = 2,   // Complex reasoning, design tasks
}
```

#### Classification Heuristics

| Complexity | Triggers | Default Thresholds |
|------------|----------|-------------------|
| **Low** | Short messages, simple greetings | < 100 characters |
| **Medium** | Medium messages, code keywords, 4+ messages | 100-500 chars, 4+ messages |
| **High** | Long messages, analysis keywords, 8+ messages | > 500 chars, 8+ messages |

#### Keywords

**High Complexity Keywords**: `explain`, `analyze`, `compare`, `design`, `architect`, `implement`, `refactor`, `debug`, `optimize`, `prove`, `derive`, `evaluate`

**Code Keywords**: `code`, `function`, `algorithm`, `class`, `struct`, `implement`, `refactor`, `debug`, `compile`, `error`, `test`

### Model Tier Mapping

Models are mapped to capability tiers based on average pricing:

```rust
fn capability_tier(tier_price: f64) -> QueryComplexity {
    if tier_price < 2.0 {
        QueryComplexity::Low      // Budget: Llama-3 8B, GPT-4o Mini
    } else if tier_price < 15.0 {
        QueryComplexity::Medium   // Mid-tier: GPT-4o, Claude 3 Sonnet
    } else {
        QueryComplexity::High     // Premium: GPT-4 Turbo, Claude 3 Opus
    }
}
```

### Configuration

#### Default Configuration

```rust
let selector = CostAwareSelector::new();
```

#### With Cost Ceiling

```rust
// Exclude models above $10/1M tokens (average)
let selector = CostAwareSelector::new().with_max_cost(10.0);
```

#### Custom Classifier

```rust
use rust_llm_api_router::domain::services::query_complexity::ClassifierConfig;

let config = ClassifierConfig {
    low_char_threshold: 50,
    medium_char_threshold: 300,
    medium_message_count: 3,
    high_message_count: 6,
    high_complexity_keywords: vec!["quantum".to_string()],
    code_keywords: vec!["rust".to_string(), "async".to_string()],
    ..Default::default()
};

let selector = CostAwareSelector::with_classifier(
    QueryClassifier::with_config(config)
);
```

### When to Use

- ✅ You want cheapest capable model **upfront**
- ✅ Queries vary predictably in complexity
- ✅ You prefer static cost budgeting
- ✅ Latency is critical (single request)
- ❌ You need guaranteed quality (use Cascading instead)

## Task-Based Routing (Issue #26)

### Purpose

Routes queries based on their **task type**, enabling model selection optimized for specific use cases like coding, reasoning, or summarization.

### Task Types

```rust
pub enum TaskType {
    General,      // General conversation, Q&A
    Chat,         // Multi-turn dialogue
    Code,         // Code generation, debugging
    Reasoning,    // Logical reasoning, math, analysis
    Summarization,// Text summarization
    Translation,  // Language translation
}
```

### Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `TaskType` enum | `src/domain/services/query_complexity.rs` | Task categories |
| `QueryClassification` struct | `src/domain/services/query_complexity.rs` | Combined complexity + task type |
| `classify_task()` method | `src/domain/services/query_complexity.rs` | Classifies task type from query |
| `classify_full()` method | `src/domain/services/query_complexity.rs` | Full classification (complexity + task) |

### Task Classification

The classifier uses keyword matching to identify task type:

| Task Type | Keywords |
|-----------|----------|
| **Code** | `code`, `function`, `class`, `implement`, `debug`, `compile`, `refactor` |
| **Reasoning** | `reason`, `prove`, `derive`, `calculate`, `logic`, `solve`, `math` |
| **Summarize** | `summarize`, `summary`, `abstract`, `condense`, `brief` |
| **Translate** | `translate`, `convert`, `language`, `spanish`, `french`, etc. |
| **Chat** | (Multi-turn conversation context) |
| **General** | Default for everything else |

### How It Works

```rust
use rust_llm_api_router::domain::services::query_complexity::{QueryClassifier, TaskType};

// Classify task type only
let task = classifier.classify_task("Write a function to calculate fibonacci");
// Returns: TaskType::Code

// Classify full query (complexity + task)
let classification = classifier.classify_full(
    vec![Message::user("Explain quantum computing in detail")]
);
// Returns: QueryClassification { complexity: High, task: Reasoning }
```

### Configuration

```rust
use rust_llm_api_router::domain::services::query_complexity::ClassifierConfig;

let config = ClassifierConfig {
    // Task-specific keywords (add to existing)
    code_keywords: vec!["rust".to_string(), "typescript".to_string()],
    reasoning_keywords: vec!["proof".to_string(), "theorem".to_string()],
    summarize_keywords: vec!["bullet points".to_string()],
    translate_keywords: vec!["translate to".to_string()],
    ..Default::default()
};
```

### When to Use

- ✅ You have different model preferences per task type
- ✅ Code queries should use specialized coding models
- ✅ Summarization tasks should use efficient models
- ❌ Simple queries where task type doesn't matter much

## Cascading Routing (Issue #24)

> ⚠️ **EXPERIMENTAL / INCOMPLETE** — The `CascadingExecutionPlan::execute()` method currently uses **simulated costs** (`cost_estimate = 1000`) and does **not** invoke the real LLM gateway. It is a stub implementation suitable for unit testing but **should not be enabled in production**. The quality evaluation logic exists (`HeuristicQualityEvaluator`) but is only tested with synthetic responses. Tracking: [#32](https://github.com/XaviCode1000/Rust-LLM-Api-Router/issues/32) (QA Audit — Threat Vector 2).

### Purpose

Starts with the cheapest tier, evaluates response quality, and escalates to more capable models **only when quality thresholds are not met**.

### Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `CascadingExecutionPlan` | `src/app/services/execution_plan/cascading.rs` | Orchestrates tier execution |
| `HeuristicQualityEvaluator` | `src/app/services/quality/evaluator.rs` | Evaluates response quality |
| `QualityGate` trait | `src/app/services/quality/evaluator.rs` | Extensible quality evaluation |
| `QualityConfig` | `src/app/services/quality/evaluator.rs` | Configuration for cascading |

### How It Works

1. **Execute Cheapest**: Sends request to lowest-cost model tier
2. **Evaluate Quality**: Checks 4 heuristic criteria
3. **Escalate if Needed**: If quality < threshold (default: 0.75), try next tier
4. **Repeat**: Continue until quality acceptable or tiers exhausted
5. **Track Costs**: Accumulates cost across all tier attempts

### Quality Evaluation

The `HeuristicQualityEvaluator` performs 4 checks:

```rust
pub struct HeuristicQualityEvaluator {
    config: QualityConfig,
}

async fn evaluate_quality(&self, response: &str) -> QualityScore {
    let mut passed = 0;
    
    // 1. Completeness - response not truncated
    if self.check_completeness(response) { passed += 1; }
    
    // 2. Length - meets minimum threshold
    if self.check_length(response) { passed += 1; }
    
    // 3. Structure - valid JSON when expected
    if self.check_structure(response) { passed += 1; }
    
    // 4. Coherence - no error patterns
    if self.check_coherence(response) { passed += 1; }
    
    QualityScore::new(passed, 4, min_quality_score)
}
```

#### Quality Checks Detail

| Check | What It Measures | Failure Condition |
|-------|------------------|-------------------|
| **Completeness** | Response not truncated | Ends with `,`, `:`, `;`, `-`, `{`, `[`, or whitespace |
| **Length** | Minimum response size | < 10 characters (configurable) |
| **Structure** | Valid JSON when expected | Unmatched `{`/`}` or `[`/`]` |
| **Coherence** | No error patterns | Contains "I cannot", "As an AI", repeated words (4+) |

### Configuration

#### Default Configuration

```rust
let config = QualityConfig::default();
// min_quality_score: 0.75
// min_response_length: 10
// max_tiers: 3
// per_tier_timeout_ms: 5000
```

#### Custom Configuration

```rust
let config = QualityConfig {
    min_quality_score: 0.85,      // Higher quality bar
    max_tiers: 4,                  // Try up to 4 tiers
    per_tier_timeout_ms: 3000,     // 3s per tier
    ..Default::default()
};
```

### Streaming Guard

Cascading is **automatically disabled** for streaming requests:

```rust
if config.stream {
    // Streaming detected - use only first tier
    // Cannot evaluate quality until stream completes
    return execute_single_tier(request);
}
```

**Why?**
- Quality can't be evaluated until stream completes
- Cascading would break real-time token delivery
- Falls back to Standard execution plan automatically

### Cost Tracking

Costs are tracked across all tier attempts:

```rust
pub struct CascadingExecutionPlan {
    total_cost_microdollars: u64,  // Accumulated cost
    tiers_attempted: u32,          // Number of tiers tried
    // ...
}

// Usage:
plan.add_cost(cost_microdollars);
let total = plan.total_cost_microdollars();
```

### When to Use

- ✅ You want to save costs but **ensure quality**
- ✅ Some queries can be handled by cheaper models
- ✅ You're willing to trade latency for cost savings
- ✅ Non-streaming requests
- ❌ Streaming requests (automatic guard prevents this)
- ❌ Latency-critical applications

## Comparison

| Aspect | Cost-Aware Routing | Cascading Routing | Task-Based Routing |
|--------|-------------------|-------------------|-------------------|
| **When** | Before request | After each tier | Before request |
| **Decision** | Static selection | Dynamic escalation | Keyword-based classification |
| **Latency** | Same as single request | Multiple tier attempts | Same as single request |
| **Cost Model** | Predictable per-request | Accumulative across tiers | Predictable per-request |
| **Quality** | Assumed from tier | Verified after response | Assumed from task type |
| **Streaming** | Compatible | Incompatible (guard) | Compatible |
| **Best For** | Budget-critical, predictable queries | Quality-critical, variable queries | Task-specific model preferences |

## Routing Configuration (Issue #29)

### Purpose

Configure routing strategies via CLI flags or environment variables for flexible control without code changes.

### CLI Flags

The router supports global CLI flags for routing configuration:

```bash
llm-router --routing-strategy auto --cascading --quality-threshold 0.85 \
  --budget-mode --max-retries 3 --timeout 60
```

| Flag | Description | Values | Default |
|------|-------------|--------|---------|
| `--routing-strategy` | Overall routing strategy | `auto`, `cost-optimized`, `cascading`, `failover`, `load-balanced` | `auto` |
| `--cascading` | Enable cascading routing | flag (sets to true) | `false` |
| `--quality-threshold` | Minimum quality score | 0.0-1.0 | `0.75` |
| `--budget-mode` | Enable budget mode | flag | `false` |
| `--max-retries` | Maximum retries per request | 1-10 | `3` |
| `--timeout` | Request timeout in seconds | 10-300 | `60` |

### Environment Variables

| Variable | Description | Values | Default |
|----------|-------------|--------|---------|
| `ROUTING_STRATEGY` | Overall routing strategy | `auto`, `cost-optimized`, `cascading`, `failover`, `load-balanced` | `auto` |
| `CASCADING_ENABLED` | Enable cascading routing | `true`, `false` | `false` |
| `CASCADING_MIN_QUALITY` | Minimum quality score | 0.0-1.0 | `0.75` |
| `CASCADING_MAX_TIERS` | Maximum tiers to try | 1-10 | `3` |
| `CASCADING_PER_TIER_TIMEOUT_MS` | Timeout per tier | 1000-30000 | `5000` |
| `BUDGET_MODE` | Enable budget mode | `true`, `false` | `false` |
| `MAX_RETRIES` | Maximum retries per request | 1-10 | `3` |
| `REQUEST_TIMEOUT_SECONDS` | Request timeout in seconds | 10-300 | `60` |

### Priority Hierarchy

Configuration priority (highest to lowest):

1. **CLI flags** (highest priority)
2. **Environment variables**
3. **Default values** (lowest priority)

```bash
# CLI flag overrides environment variable
export CASCADING_ENABLED=false
llm-router --cascading  # Uses true (from CLI)
```

### Strategy Options

| Strategy | Description | Use Case |
|---------|-------------|----------|
| `auto` | Automatically selects best strategy based on request | Default, general use |
| `cost-optimized` | Uses Cost-Aware selector for static model selection | Budget-critical |
| `cascading` | Uses CascadingExecutionPlan for quality-based escalation | Quality-critical |
| `failover` | Sequential fallback on failure | Reliability |
| `load-balanced` | Health-weighted distribution | High throughput |

### Examples

```bash
# Enable cascading with environment variable
export ROUTING_STRATEGY=cascading
export CASCADING_MIN_QUALITY=0.8
./target/release/llm-router

# Enable cascading with CLI flag
llm-router --routing-strategy cascading --quality-threshold 0.85

# Budget mode with cost-optimized
llm-router --routing-strategy cost-optimized --budget-mode

# High reliability with failover
llm-router --routing-strategy failover --max-retries 5 --timeout 120
```

## Integration with Execution Plans

### Plan Types

Both strategies extend the `ExecutionPlanType` enum:

```rust
pub enum ExecutionPlanType {
    Standard,      // Single account
    Failover,      // Sequential fallback
    LoadBalanced,  // Health-weighted distribution
    CostOptimized, // Cheapest provider selection
    Cascading,     // Quality-based escalation (Issue #24)
}
```

### Model Selector Integration

The `CostAwareSelector` (Issue #23) can be used by `CostOptimized` and `Cascading` plan types:

```rust
// CostOptimized uses selector for pre-request selection
let selector = CostAwareSelector::new();
let model = selector.select(&request, &available_models)?;

// Cascading uses tier ordering based on cost
let tiers = sort_accounts_by_cost(accounts);
```

## Examples

### Example 1: Cost-Aware Selection

```rust
use rust_llm_api_router::domain::services::model_selector::CostAwareSelector;

let selector = CostAwareSelector::new();

let request = ChatRequest::new(
    "gpt-4",
    vec![Message::user("Explain quantum computing in detail")]
);

let models = vec![
    Model::with_pricing("llama-3-8b", "Llama 3 8B", "groq", 
        ModelPricing::new(0.05, 0.10)),
    Model::with_pricing("gpt-4o", "GPT-4o", "openai",
        ModelPricing::new(5.0, 15.0)),
    Model::with_pricing("gpt-4-turbo", "GPT-4 Turbo", "openai",
        ModelPricing::new(10.0, 30.0)),
];

// "Explain" keyword + long query → High complexity
// Selects cheapest High-tier model: gpt-4-turbo
let selected = selector.select(&request, &models)?;
```

### Example 2: Cascading Execution

```rust
use rust_llm_api_router::app::services::quality::evaluator::QualityConfig;

let quality_config = QualityConfig {
    min_quality_score: 0.75,
    max_tiers: 3,
    ..Default::default()
};

let plan = CascadingExecutionPlan::new(
    context,
    accounts,      // Sorted by cost (cheapest first)
    pricing,
    model_ids,
    quality_config,
    quality_gate,
);

// Execute with cascading logic
let result = plan.execute(config, response_text, tokens_used);

match result {
    ExecutionResult::Success { tier_used, quality_score, .. } => {
        println!("Success with tier {} (quality: {:.2})", 
            tier_used, quality_score.unwrap_or(0.0));
    }
    ExecutionResult::Failure { .. } => {
        println!("All tiers exhausted");
    }
}
```

## Performance Considerations

### Latency Impact

- **Cost-Aware**: No additional latency (pre-request selection)
- **Cascading**: Up to N × tier_timeout if all tiers attempted

### Cost Optimization

- **Cost-Aware**: Saves money by avoiding expensive models
- **Cascading**: Saves money by starting cheap, only escalating when needed

### Recommended Settings

| Use Case | Strategy | Quality Threshold | Max Tiers |
|----------|----------|-------------------|-----------|
| Simple Q&A | Cost-Aware | N/A | N/A |
| Code generation | Cascading | 0.8 | 3 |
| Creative writing | Cascading | 0.7 | 2 |
| Technical analysis | Cascading | 0.85 | 4 |
| Real-time chat | Cost-Aware | N/A | N/A |

## See Also

- [Execution Plan Module](../src/app/services/execution_plan/README.md) - Detailed execution plan documentation
- [Architecture](architecture.md) - Overall system architecture
- [API Reference](api.md) - API endpoints and usage
- [Cascading Routing Exploration](cascading-routing-exploration.md) - Technical exploration of cascading routing design