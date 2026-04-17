# Code Processes — GitNexus Analysis

> Generated from GitNexus code knowledge graph analysis (fresh re-index 2026-04-17).

## Overview

This document lists the key execution flows and processes detected in the codebase, organized by functional area. These processes were automatically extracted using GitNexus graph analysis.

## Key Execution Flows

### Chat Handler Flow

| Process | Summary | Steps |
|---------|---------|-------|
| `proc_107` | Test_list_models_no_accounts → New | 4 |
| `proc_126` | Test_cascading_current_tier → Health_score | 4 |
| `proc_127` | Test_cascading_cost_tracking → Health_score | 4 |
| `proc_128` | Execute → Health_score | 4 |
| `proc_161` | Test_cascading_execution_plan_new → Health_score | 4 |

### Provider Management

| Process | Summary | Steps |
|---------|---------|-------|
| `proc_231` | Test_remove_provider_success → AddProviderArgs | 3 |
| `proc_234` | Test_remove_provider_from_multiple → AddProviderArgs | 3 |
| `proc_237` | Test_enable_provider_success → AddProviderArgs | 3 |
| `proc_240` | Test_enable_already_enabled_provider → AddProviderArgs | 3 |
| `proc_243` | Test_disable_provider_success → AddProviderArgs | 3 |
| `proc_246` | Test_disable_already_disabled_provider → AddProviderArgs | 3 |
| `proc_249` | Test_validate_provider_success → AddProviderArgs | 3 |
| `proc_252` | Test_validate_disabled_provider → AddProviderArgs | 3 |
| `proc_255` | Test_validate_provider_unreachable_url → AddProviderArgs | 3 |

### Account Management

| Process | Summary | Steps |
|---------|---------|-------|
| `proc_269` | Test_remove_account_success → AddAccountArgs | 3 |
| `proc_270` | Test_remove_account_from_multiple → AddAccountArgs | 3 |
| `proc_271` | Test_set_priority_success → AddAccountArgs | 3 |
| `proc_272` | Test_set_priority_negative_value → AddAccountArgs | 3 |
| `proc_273` | Test_validate_account_success → AddAccountArgs | 3 |
| `proc_274` | Test_validate_account_short_key → AddAccountArgs | 3 |

### Cascading Execution

| Process | Summary | Steps |
|---------|---------|-------|
| `proc_223` | Test_cascading_current_tier → Estimate_cost | 3 |
| `proc_224` | Test_cascading_current_tier → With_execution_order | 3 |
| `proc_225` | Test_cascading_cost_tracking → Estimate_cost | 3 |

## Process Types

| Type | Description |
|------|-------------|
| **intra_community** | Processes within a single functional area |
| **cross_community** | Processes spanning multiple areas |

## Key Symbols

### Files

- `src/interfaces/handlers/chat_handler.rs` — Main chat endpoint
- `src/app/router/llm_router.rs` — Router implementation
- `src/app/services/execution_plan/plan.rs` — Execution plan logic
- `src/app/services/execution_plan/planner.rs` — Planner configuration
- `src/app/services/execution_plan/cascading.rs` — Cascading execution
- `src/infrastructure/metrics.rs` — Metrics collection

### Key Structs

- `LlmRouter` — Main router
- `LlmRouterConfig` — Router configuration
- `ExecutionPlanImpl` — Execution plan implementation
- `ExecutionPlanBuilder` — Plan builder
- `ExecutionPlannerConfig` — Planner configuration

## See Also

- [Architecture](architecture.md)
- [docs/routing.md](routing.md) — Routing strategies
- [GitNexus documentation](https://github.com/XaviCode1000/Rust-LLM-Api-Router)

---

**Note**: This document was auto-generated from GitNexus graph analysis. For real-time queries, use the GitNexus CLI.