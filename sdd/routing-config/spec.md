# Specification: CLI and Configuration Options for Routing Strategies (Issue #29)

## Requirements

### REQ-1: Routing Strategy CLI Flag
The CLI SHALL accept `--routing-strategy <strategy>` where strategy is one of: `auto`, `cost-optimized`, `cascading`, `failover`, `load-balanced`. Default: `auto`.

### REQ-2: Cascading Configuration
The CLI SHALL accept:
- `--cascading` — Enable cascading routing
- `--quality-threshold <0.0-1.0>` — Minimum quality score (default: 0.75)

### REQ-3: Budget Mode CLI Flag
The CLI SHALL accept `--budget-mode` to enable cost-aware model selection.

### REQ-4: Retry and Timeout Configuration
The CLI SHALL accept:
- `--max-retries <n>` — Maximum retries per request (default: 3)
- `--timeout <seconds>` — Request timeout (default: 60)

### REQ-5: Environment Variables
The system SHALL support environment variables that override defaults:
- `ROUTING_STRATEGY`, `CASCADING_ENABLED`, `CASCADING_MIN_QUALITY`, `CASCADING_MAX_TIERS`, `CASCADING_PER_TIER_TIMEOUT_MS`, `BUDGET_MODE`, `MAX_RETRIES`, `REQUEST_TIMEOUT_SECONDS`

### REQ-6: Configuration Priority
Configuration priority SHALL be: CLI flags > Environment variables > Built-in defaults.

### REQ-7: Validation
- Quality threshold MUST be between 0.0 and 1.0
- Routing strategy MUST be a valid enum value
- Max retries MUST be >= 0
- Timeout MUST be > 0

### REQ-8: Logging
Every request SHALL log which routing strategy was used, including: request_id, strategy name, number of accounts in plan.

### REQ-9: Backward Compatibility
Existing environment variables (`EXECUTION_PLAN_TYPE`, `EXECUTION_AUTO_SELECTION`, `EXECUTION_MAX_ACCOUNTS`, `EXECUTION_MAX_RETRIES`, `EXECUTION_TIMEOUT_SECONDS`) SHALL continue to work.

### REQ-10: Rich Help
All routing-related CLI flags SHALL include examples in their help text.

## Scenarios

### Scenario 1: Default Auto Strategy
**Given** user runs `llm-router --port 8080` (no routing flags)
**When** server starts
**Then** routing strategy is `auto` (planner decides based on context)

### Scenario 2: Force Cascading via CLI
**Given** user runs `llm-router --routing-strategy cascading --quality-threshold 0.85`
**When** server starts
**Then** all requests use cascading with quality threshold 0.85

### Scenario 3: Budget Mode via Env Var
**Given** user runs `BUDGET_MODE=true llm-router`
**When** server starts
**Then** budget mode is enabled (cost-optimized routing)

### Scenario 4: CLI Overrides Env
**Given** user runs `CASCADING_ENABLED=true llm-router --routing-strategy failover`
**When** server starts
**Then** routing strategy is `failover` (CLI wins over env)

### Scenario 5: Invalid Quality Threshold
**Given** user runs `llm-router --quality-threshold 1.5`
**When** CLI parses arguments
**Then** error: "Quality threshold must be between 0.0 and 1.0"

### Scenario 6: Invalid Strategy
**Given** user runs `llm-router --routing-strategy invalid`
**When** CLI parses arguments
**Then** error: "Invalid routing strategy 'invalid'. Valid values: auto, cost-optimized, cascading, failover, load-balanced"

### Scenario 7: Logging Strategy Used
**Given** server running with `--routing-strategy cascading`
**When** request is processed
**Then** log shows: `Routing request request_id=abc123 strategy=cascading accounts=3`

### Scenario 8: Help with Examples
**Given** user runs `llm-router --help`
**When** help is displayed
**Then** output includes routing strategy options with examples
