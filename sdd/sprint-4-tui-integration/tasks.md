# Tasks: Sprint 4 - TUI Integration with LlmRouter

## Phase 1: Foundation (Hot-Reload Infrastructure)

- [ ] 1.1 Add RwLock<Vec<ProviderConfig>> providers field to LlmRouter struct (src/app/router/llm_router.rs)
- [ ] 1.2 Add reload_accounts(new_accounts) method with short-lived write lock (no I/O while locked)
- [ ] 1.3 Add #[cfg(feature = "tui")] reload method that syncs providers to TuiState after reload

## Phase 2: Telemetry Injection

- [ ] 2.1 Add telemetry_update() method in LlmRouter using send_modify (src/app/router/llm_router.rs)
- [ ] 2.2 Call telemetry_update() after each provider response in execute_with_fallback
- [ ] 2.3 Update provider metrics: latency_ms, requests_success/requests_failed, circuit_breaker state
- [ ] 2.4 Update global stats: requests_total counter, moving average latency calculation
- [ ] 2.5 Append routing result to log_buffer with provider_id

## Phase 3: TuiAction Processor

- [ ] 3.1 Add #[cfg(feature = "tui")] section in main.rs for async task spawning
- [ ] 3.2 Spawn async task with mpsc receiver loop for TuiAction commands
- [ ] 3.3 Handle AddAccount: persist to AccountRepository → call reload → log confirmation
- [ ] 3.4 Handle RemoveAccount: delete from repository → call reload → log confirmation
- [ ] 3.5 Handle ToggleProvider: update provider enabled state → log result
- [ ] 3.6 Use try_send for sending responses back, handle channel closed gracefully

## Phase 4: Signal Handling

- [ ] 4.1 Use tokio::signal::ctrl_c() to detect SIGINT/SIGTERM
- [ ] 4.2 Send shutdown signal to TuiEngine for terminal cleanup
- [ ] 4.3 Wait for cleanup before exit (proper shutdown sequence)
- [ ] 4.4 Ensure panic hook restores terminal in case of crash

## Phase 5: Feature Gating & Verification

- [ ] 5.1 Verify all TUI code has #[cfg(feature = "tui")] in main.rs
- [ ] 5.2 Test build without TUI: cargo check (should pass with no default features)
- [ ] 5.3 Test build with TUI: cargo check --features tui
- [ ] 5.4 Run cargo fmt and cargo clippy -D warnings
- [ ] 5.5 Run tests: cargo nextest run --test-threads 2

## Implementation Notes

### Hot-Reload Pattern
- Use tokio::sync::RwLock for provider configs (Arc<RwLock<Vec<ProviderConfig>>>)
- Short-lived write locks: acquire lock → copy data → release lock → do I/O
- After reload: call tui_state_tx.send_modify() to update TuiState

### Telemetry Updates
- Use watch::Sender<TuiState>::send_modify() for in-place updates (not send())
- Calculate moving average latency: (old_avg * n + new_latency) / (n + 1)
- Log entries: include timestamp, level, message, provider_id

### TuiAction Processing
- Spawn as separate tokio task with mpsc receiver
- Handle channel full: use try_send() and log warning if full
- Handle channel closed: break loop gracefully

### Signal Handling
- Run ctrl_c() in spawned task, signal to TUI thread via action channel
- TUI engine catches Quit action, cleans up terminal, exits
- Main thread waits for TUI handle to complete