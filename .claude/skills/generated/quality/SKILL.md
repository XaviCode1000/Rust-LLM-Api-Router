---
name: quality
description: "Skill for the Quality area of Rust-LLM-Api-Router. 11 symbols across 1 files."
---

# Quality

11 symbols | 1 files | Cohesion: 66%

## When to Use

- Working with code in `src/`
- Understanding how new work
- Modifying quality-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/app/services/quality/evaluator.rs` | new, default, test_heuristic_quality_evaluator_new, test_check_completeness, test_check_length (+6) |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/app/services/quality/evaluator.rs:27`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/app/services/quality/evaluator.rs` | 27 |
| `default` | Function | `src/app/services/quality/evaluator.rs` | 61 |
| `test_heuristic_quality_evaluator_new` | Function | `src/app/services/quality/evaluator.rs` | 287 |
| `test_check_completeness` | Function | `src/app/services/quality/evaluator.rs` | 296 |
| `test_check_length` | Function | `src/app/services/quality/evaluator.rs` | 322 |
| `test_check_structure` | Function | `src/app/services/quality/evaluator.rs` | 337 |
| `test_check_coherence` | Function | `src/app/services/quality/evaluator.rs` | 357 |
| `create_test_account` | Function | `src/app/services/quality/evaluator.rs` | 280 |
| `test_evaluate_quality_good_response` | Function | `src/app/services/quality/evaluator.rs` | 379 |
| `test_evaluate_quality_bad_response` | Function | `src/app/services/quality/evaluator.rs` | 395 |
| `test_evaluate_quality_partial_response` | Function | `src/app/services/quality/evaluator.rs` | 412 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Tests | 3 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "quality"})` — find related execution flows
3. Read key files listed above for implementation details
