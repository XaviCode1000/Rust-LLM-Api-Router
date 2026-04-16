# Proposal: Sprint 4 - TUI Integration with LlmRouter

## Intent

Connect the TUI (Sprint 3) to the LlmRouter for real-time telemetry. Currently TuiState exists but is unused - the watch channel is wired but no telemetry data flows. This enables live provider metrics, latency tracking, and account hot-reload from the TUI.

## Scope

### In Scope
1. Telemetry injection in LlmRouter route/dispatch method using `send_modify`
2. TuiAction processor loop in main.rs (mpsc receiver handling AddAccount/RemoveAccount)
3. Hot-reload pattern with RwLock for providers (short-lived write locks)
4. Signal handling (SIGINT/SIGTERM) with terminal cleanup before exit
5. Feature gating (`#[cfg(feature = "tui")]`) for all TUI code

### Out of Scope
- OAuth/PKCE authentication flow (covered in secure-keyring)
- Provider config editing UI beyond enable/disable toggle
- Persistent TUI settings between sessions

## Capabilities

### New Capabilities
- `tui-telemetry`: Real-time provider metrics streamed from LlmRouter to TUI dashboard
- `tui-hot-reload`: Account changes from TUI trigger immediate provider reload

### Modified Capabilities
- `tui-interactive-forms`: Extends with account persistence and hot-reload on add/remove

## Approach

**Telemetry Injection:**
- In `llm_router.rs` route_request method, after each provider response:
  - Use `tui_state_tx.send_modify(|state| { ... })` for in-place updates
  - Update provider latency, success/fail counters, circuit breaker status
  - Update global stats (requests_total, moving average latency)
  - Append routing result to log_buffer

**TuiAction Processor:**
- In `main.rs`, spawn async task with mpsc receiver loop:
  - AddAccount → persist to AccountRepository, send confirmation log
  - RemoveAccount → remove from repository, send confirmation log
  - ToggleProvider → update provider enabled state
  - Send results back to TuiState via watch channel

**Hot-Reload Pattern:**
- Wrap provider configs in RwLock in LlmRouter
- Short-lived write locks only for mutation (no I/O while locked)
- After reload, sync providers to TuiState

**Signal Handling:**
- Use `tokio::signal::ctrl_c()` in main.rs
- Send signal to TuiEngine for terminal cleanup before exit

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/app/router/llm_router.rs` | Modified | Add telemetry injection in route/dispatch methods |
| `src/main.rs` | Modified | Add TuiAction processor loop, signal handling |
| `src/presentation/tui/engine.rs` | Modified | Add shutdown signal receiver for terminal cleanup |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Watch channel contention | Medium | Use `send_modify` (in-place) not `send` (replace) |
| Deadlock on RwLock | Low | Short-lived write locks, no I/O while locked |
| Circular coupling | Low | Router holds only Sender<TuiState>, not UI |
| UI thread death | Medium | Ignore errors if channel `is_closed()` |
| Telemetry panic | Low | Wrap in block that ignores SendError |

## Rollback Plan

1. Remove telemetry injection from llm_router.rs
2. Remove TuiAction processor from main.rs
3. Disable signal handling in engine.rs
4. TUI still compiles but receives no updates (graceful degradation)
5. Feature flag `tui` still works independently

## Dependencies

- Sprint 3 TUI infrastructure (TuiState, TuiAction, watch/mpsc channels)
- AccountRepository trait for persistence
- RwLock utilities from tokio

## Success Criteria

- [ ] `tui` feature compiles: `cargo check --features tui`
- [ ] No-tui builds pass: `cargo check` (no default features)
- [ ] Telemetry updates appear in TUI dashboard during requests
- [ ] AddAccount persists and triggers hot-reload
- [ ] SIGINT/SIGTERM cleanly exits TUI (no stuck terminal)
- [ ] No `.unwrap()` in production telemetry code (use `ok()` or `if let Err`)
