# Delta Specification for Cascading Routing

## Purpose
Implement cascading routing strategy that starts with the cheapest model, evaluates response quality, and escalates to more capable models only when quality thresholds are not met. This follows the RouteLLM approach to achieve significant cost reductions.

## ADDED Requirements

### Requirement: QualityGate Trait
The system MUST provide a QualityGate trait for evaluating LLM response quality without requiring additional LLM calls.

#### Scenario: Quality evaluation interface
- GIVEN a response from an LLM execution
- WHEN the QualityGate trait is implemented for a quality evaluator
- THEN it MUST provide a method to evaluate response quality
- AND return a quality score between 0.0 and 1.0
- AND indicate whether the quality meets the required threshold

#### Scenario: Synchronous evaluation
- GIVEN a QualityGate implementation
- WHEN evaluating response quality
- THEN the evaluation MUST be synchronous (no additional LLM calls)
- AND complete within a reasonable time limit (<100ms)
- AND not depend on external services

### Requirement: HeuristicQualityEvaluator Implementation
The system SHALL provide a default HeuristicQualityEvaluator that implements the QualityGate trait using configurable heuristics.

#### Scenario: Heuristic-based quality evaluation
- GIVEN a HeuristicQualityEvaluator with configured thresholds
- WHEN evaluating a response with specific characteristics
- THEN it MUST calculate a quality score based on:
  - Response length appropriateness
  - Presence of expected patterns/keywords
  - Absence of error indicators
  - Coherence and completeness heuristics
- AND return a score between 0.0 and 1.0
- AND correctly determine if quality meets threshold

#### Scenario: Configurable thresholds
- GIVEN a HeuristicQualityEvaluator with quality_threshold = 0.7
- WHEN evaluating a high-quality response (score 0.8)
- THEN it MUST indicate quality meets threshold
- GIVEN the same evaluator with a low-quality response (score 0.5)
- THEN it MUST indicate quality does NOT meet threshold

### Requirement: QualityConfig Structure
The system SHALL provide a QualityConfig structure to configure cascading behavior.

#### Scenario: Quality configuration
- GIVEN a QualityConfig instance
- WHEN setting configuration parameters
- THEN it MUST support:
  - quality_threshold: minimum quality score to accept response (default 0.7)
  - max_tiers: maximum number of models to try in cascade (default 3)
  - per_tier_timeout: timeout seconds per tier attempt (default 15)
  - enable_cost_tracking: whether to accumulate costs across tiers (default true)
- AND provide sensible defaults for all parameters

#### Scenario: Configuration validation
- GIVEN a QualityConfig with invalid parameters
- WHEN creating the config with quality_threshold < 0.0 or > 1.0
- THEN it MUST clamp the value to valid range [0.0, 1.0]
- GIVEN max_tiers < 1
- THEN it MUST default to 1
- GIVEN per_tier_timeout < 1
- THEN it MUST default to 5 seconds

### Requirement: CascadingExecutionPlan Implementation
The system SHALL provide a CascadingExecutionPlan that implements the ExecutionPlan trait following the existing delegation pattern.

#### Scenario: Cascading plan structure
- GIVEN a CascadingExecutionPlan instance
- WHEN inspecting its structure
- THEN it MUST contain an inner ExecutionPlanImpl
- AND delegate all ExecutionPlan trait methods to the inner implementation
- AND maintain a reference to QualityGate implementation
- AND hold QualityConfig for cascading behavior

#### Scenario: Plan type identification
- GIVEN a CascadingExecutionPlan
- WHEN calling plan_type()
- THEN it MUST return ExecutionPlanType::Cascading

#### Scenario: Account preparation for cascading
- GIVEN a CascadingExecutionPlan being built
- WHEN the planner prepares accounts
- THEN it MUST sort accounts by estimated cost (cheapest first)
- AND ensure each PlannedAccount has a model_id set
- AND limit accounts to max_tiers from QualityConfig

### Requirement: Streaming Request Guard
The system MUST prevent cascading execution plans from being used with streaming requests.

#### Scenario: Streaming detection
- GIVEN an ExecutionContext with streaming enabled (stream: true in request_params)
- WHEN the ExecutionPlanner selects a plan type
- THEN it MUST NOT return ExecutionPlanType::Cascading
- AND fallback to an appropriate non-streaming plan type (e.g., Standard)
- AND log a warning about cascading incompatibility with streaming

#### Scenario: Planning-time validation
- GIVEN a CascadingExecutionPlan being constructed
- WHEN the context indicates streaming request
- THEN the plan construction MUST fail with a clear error
- AND indicate that cascading is incompatible with streaming

### Requirement: Multi-tier Execution Logic
The system SHALL implement the cascading execution logic that tries tiers until quality is met or all tiers exhausted.

#### Scenario: Successful early tier
- GIVEN a CascadingExecutionPlan with 3 tiers configured
- WHEN the first tier (cheapest) produces a response meeting quality threshold
- THEN the execution MUST stop after first tier
- AND return the outcome from the successful tier
- AND NOT attempt subsequent tiers

#### Scenario: Quality-based escalation
- GIVEN a CascadingExecutionPlan with 3 tiers configured
- WHEN the first tier produces response below quality threshold
- THEN the execution MUST proceed to second tier
- GIVEN the second tier produces response meeting quality threshold
- THEN the execution MUST stop after second tier
- AND return outcome from second tier

#### Scenario: Full cascade exhaustion
- GIVEN a CascadingExecutionPlan with 3 tiers configured
- WHEN all tiers produce responses below quality threshold
- THEN the execution MUST attempt all configured tiers
- AND return the outcome from the final tier (even if low quality)
- AND set an appropriate error message indicating quality thresholds not met

#### Scenario: Tier failure handling
- GIVEN a CascadingExecutionPlan with 3 tiers configured
- WHEN the first tier fails completely (timeout, error)
- THEN the execution MUST proceed to second tier
- GIVEN the second tier also fails
- THEN the execution MUST proceed to third tier
- AND return outcome from whichever tier succeeded last
- AND if all tiers fail, return error from final tier attempt

### Requirement: Cost Tracking Across Tiers
The system SHALL track cumulative cost across all tiers attempted in a cascading execution.

#### Scenario: Cost accumulation
- GIVEN a CascadingExecutionPlan with cost tracking enabled
- WHEN executing across multiple tiers
- THEN it MUST accumulate estimated costs from each tier attempt
- AND make total cost available in the final ExecutionOutcome
- AND include cost metadata showing per-tier costs and total

#### Scenario: Cost tracking disabled
- GIVEN a CascadingExecutionPlan with cost tracking disabled
- WHEN executing across tiers
- THEN it MUST NOT accumulate or track costs
- AND final outcome MUST NOT include cost information

### Requirement: Integration with ExecutionPlanner
The system SHALL integrate CascadingExecutionPlan into the existing ExecutionPlanner service.

#### Scenario: Plan builder extension
- GIVEN an ExecutionPlanBuilder with account repository
- WHEN requesting to build a cascading plan
- THEN it MUST provide a build_cascading method
- AND accept context, accounts, and optional QualityConfig
- AND return a properly constructed CascadingExecutionPlan

#### Scenario: Planner plan type selection
- GIVEN an ExecutionPlanner with cost optimization enabled
- WHEN context indicates preference for cost optimization
- AND no higher priority plan type is specified
- THEN the planner MAY select ExecutionPlanType::Cascading
- WHEN quality evaluation components are available
- AND cascading is appropriate for the request type (non-streaming)

#### Scenario: Planner build plan dispatch
- GIVEN an ExecutionPlanner processing a context
- WHEN the selected plan_type is ExecutionPlanType::Cascading
- THEN it MUST dispatch to the cascading plan builder
- AND pass appropriate accounts and configuration
- AND handle any builder errors appropriately

## Modified Requirements

### Requirement: ExecutionPlanType extensions
The ExecutionPlanType enum already includes Cascading variant, but its behavior descriptions NEED to be updated to reflect cascading-specific capabilities.

#### Scenario: Cost optimization identification
- GIVEN ExecutionPlanType::Cascading
- WHEN calling is_cost_optimized()
- THEN it MUST return true (already implemented)
- BECAUSE cascading aims to minimize cost by starting with cheapest options

#### Scenario: Cascading capability identification
- GIVEN ExecutionPlanType::Cascading
- WHEN calling supports_cascading()
- THEN it MUST return true (already implemented)
- AND this identifies the plan's ability to perform quality-based escalation

### Requirement: PlannedAccount model_id field
The PlannedAccount struct already includes model_id field, but its usage in cascading context NEEDS clarification.

#### Scenario: Model ID utilization
- GIVEN a PlannedAccount in a cascading execution plan
- WHEN the plan is executed
- THEN the model_id field MUST be used to identify which model to request
- AND this enables routing to different model tiers based on cost/quality tradeoffs
- BECAUSE different accounts may serve different models with varying capabilities and costs

## REMOVED Requirements
(None - this is additive functionality)

## Implementation Notes

### Error Cases
- GIVEN a cascading execution where all tiers fail
- WHEN no successful outcome is obtained
- THEN the plan MUST set an error message indicating "All cascading tiers failed"
- AND include details about the final failure encountered

- GIVEN a cascading execution attempted on a streaming request
- WHEN the plan is being constructed
- THEN it MUST fail with error "Cascading execution plan is incompatible with streaming requests"

### Configuration Precedence
- GIVEN conflicting quality configurations
- WHEN QualityConfig is provided to builder
- THEN builder-provided config MUST override context planning options
- AND context planning options MUST override ExecutionPlanner defaults
- AND ExecutionPlanner defaults provide fallback values

### Quality Evaluation Extensibility
- GIVEN the QualityGate trait
- WHEN implementing custom quality evaluators
- THEN implementers MAY use any synchronous evaluation technique
- AND MUST implement the evaluate_quality method returning (score, meets_threshold)
- AND SHOULD keep evaluation fast (<50ms typical) to maintain latency benefits