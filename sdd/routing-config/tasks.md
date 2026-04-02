# Tasks: CLI and Configuration Options for Routing Strategies (Issue #29)

## Implementation Tasks

### Phase 1: RoutingConfig Module
- [ ] **T1: Create `src/config/routing.rs`** — RoutingStrategy enum, RoutingConfig struct, from_cli_and_env()
- [ ] **T2: Update `src/config/mod.rs`** — Re-export RoutingConfig
- [ ] **T3: Add CLI flags to `Cli` struct** — routing_strategy, cascading, quality_threshold, budget_mode, max_retries, timeout
- [ ] **T4: Add rich help text** — Examples and strategy descriptions in --help

### Phase 2: Integration
- [ ] **T5: Wire RoutingConfig to ExecutionPlanner** — Update planner.rs to use RoutingConfig
- [ ] **T6: Wire QualityConfig** — Connect cascading_min_quality, max_tiers, per_tier_timeout to planner
- [ ] **T7: Update LlmRouter** — Pass routing config, add strategy logging
- [ ] **T8: Update main.rs** — Initialize RoutingConfig from CLI + env

### Phase 3: Verification
- [ ] **T9: Verify compilation** — `cargo check` passes
- [ ] **T10: Run tests** — All tests pass
- [ ] **T11: Format and lint** — `cargo fmt --check` and `cargo clippy -- -D warnings` clean
- [ ] **T12: Update docs** — Update `docs/cli.md` and `docs/routing.md`
- [ ] **T13: Manual testing** — Test all CLI flags interactively

## Dependencies

```
T1 → T2 → T3 → T4
T5 (after T1-T2)
T6 (after T1, T5)
T7 (after T5-T6)
T8 (after T1-T7)
T9-T13 (after T8)
```

## Notes

- All changes are additive — no breaking changes to existing behavior
- Existing env vars (EXECUTION_PLAN_TYPE, etc.) must continue to work
- CLI flags override environment variables
- Quality threshold validation: 0.0-1.0
- Strategy validation: auto, cost-optimized, cascading, failover, load-balanced
