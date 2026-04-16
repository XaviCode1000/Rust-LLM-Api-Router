# Design: Sprint 4 - TUI Integration with LlmRouter

## Technical Approach

Connect LlmRouter to the existing TUI infrastructure (from Sprint 3) by injecting telemetry after each provider response. The router holds a `Option<watch::Sender<TuiState>>` and uses fire-and-forget telemetry with `send_modify()` pattern to avoid blocking. Main.rs spawns a TuiAction processor loop that handles AddAccount/RemoveAccount actions via mpsc channel.

## Architecture Decisions

### Decision: Telemetry Injection Pattern

**Choice**: Use `watch::Sender<TuiState>` with `send_modify()` instead of `send()`. The watch channel is already Arc-wrapped in state.rs, so we send closures to modify state in-place rather than replacing the entire state.

**Alternatives considered**:
- `send()` with full state replacement — Creates new Arcs on every update, higher allocation overhead
- Dedicated mpsc channel — More complex, requires more infrastructure
- Direct mutex-protected state — Higher contention, blocks router thread

**Rationale**: The existing TuiState already uses Arc-wrapped fields for efficient cloning. Using `send_modify` (or equivalent in-place update pattern) minimizes allocations and avoids blocking. Fire-and-forget semantics mean router continues if UI dies.

### Decision: TuiAction Processing Location

**Choice**: Process TuiAction in main.rs as a spawned async task with mpsc receiver.

**Alternatives considered**:
- Process in AppState — Couples handler logic to HTTP state, violates separation
- Process in LlmRouter — Adds non-routing concerns to router
- Process in separate service — Additional layer, complexity for simple operations

**Rationale**: main.rs is the entry point where app lifetime is managed. Spawning a task there keeps concerns separated and allows graceful shutdown coordination.

### Decision: Hot-Reload Pattern for Providers

**Choice**: Wrap provider configs in `Arc<RwLock<Vec<ProviderConfig>>>` in LlmRouter. Short-lived write locks only — lock → update → unlock immediately with no I/O.

**Alternatives considered**:
- Channel-based reload — Requires sender in router, more complex
- Mutex-protected mutable reference — Same as RwLock but exclusive
- Clone-on-reload — Simpler but higher memory churn on frequent changes

**Rationale**: RwLock allows concurrent reads (common case) and short-exclusive writes. No I/O while locked prevents deadlocks. The provider list is small enough that cloning is negligible overhead.

### Decision: Signal Handling

**Choice**: Use `tokio::signal::ctrl_c()` in main.rs to detect SIGINT, send signal to TuiEngine via dedicated channel for terminal cleanup.

**Alternatives considered**:
- panic hook only — Doesn't clean up TUI properly on graceful SIGINT
- atexit handler — Runs too late, can't render
- crossterm'sbuilt-in — Not integrated with async tokio

**Rationale**: TUI runs in its own thread (blocking). The async main task needs to detect ctrl_c and signal the TUI thread to exit cleanly. Engine already has cleanup code at end of run() — we just need to trigger it.

### Decision: Feature Gating

**Choice**: Gate all TUI code with `#[cfg(feature = "tui")]`. Cargo.toml has `tui = []` as optional feature (not default).

**Alternatives considered**:
- Compile-time feature detection — Doesn't work for optional UI
- Runtime feature toggle — Adds runtime overhead
- Separate binary — Would require code duplication

**Rationale**: Follows existing pattern in codebase. Ratatui and crossterm dependencies already exist unconditionally in Cargo.toml (line 65-66).

## Data Flow

```
┌────────────────────���────────────────────────────────────────────────┐
│                        main.rs                                     │
│  ┌─────────────────┐     ┌──────────────────┐                    │
│  │ AppState        │────▶│ LlmRouter         │                    │
│  │ (HTTP server)   │     │ (route_request)  │                    │
│  └─────────────────┘     └────────┬─────────┘                    │
│                                  │                                │
│                    ┌─────────────▼─────────────┐                 │
│                    │ Telemetry Injection      │                 │
│                    │ (after provider response) │                 │
│                    │ tui_state_tx.send_modify() │                 │
│                    └─────────────────────────┘                 │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ TuiAction Processor Task (spawned)                         │ │
│  │ - mpsc::receiver loop                                       │ │
│  │ - AddAccount → AccountRepository → router.reload()          │ │
│  │ - RemoveAccount → AccountRepository → router.reload()       │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Signal Handler (spawned)                                    │ │
│  │ - tokio::signal::ctrl_c()                                    │ │
│  │ - signal TuiEngine shutdown                                 │ │
│  └──────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                    TUI Thread (blocking)                          │
│  ┌─────────────────┐     ┌──────────────────┐                  │
│  │ watch::Receiver │────▶│ TuiEngine::run   │                  │
│  │ (TuiState)     │     │ (ratatui)       │                  │
│  └─────────────────┘     └──────────────────┘                  │
└───────────────────────────────────────────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/app/router/llm_router.rs` | Modify | Add telemetry injection calls after provider response |
| `src/main.rs` | Modify | Add TuiAction processor task, signal handling, TUI thread spawn |
| `src/presentation/tui/engine.rs` | Modify | Add shutdown signal receiver for clean exit |

### Details

#### 1. src/app/router/llm_router.rs

- In `execute_with_fallback()`: After successful provider response, call telemetry injection helper
- Add helper methods (gated with `#[cfg(feature = "tui")]`):
  - `fn inject_telemetry_success(&self, provider_id, latency_ms)`
  - `fn inject_telemetry_failure(&self, provider_id, error)`

#### 2. src/main.rs

- Add `tui_action_processor()` async function with mpsc receiver
- Add `signal_handler()` async function for ctrl_c detection  
- Add TUI thread spawn (std::thread::spawn with action_tx sender passed in)

#### 3. src/presentation/tui/engine.rs

- Add parameter to `run()` for shutdown signal receiver
- Replace `running = false` loop break with signal-driven exit

## Interfaces / Contracts

### Telemetry Injection Helper

```rust
#[cfg(feature = "tui")]
fn inject_telemetry_success(&self, provider_id: &str, latency_ms: u64) {
    if let Some(ref tx) = self.tui_state_tx {
        let _ = tx.send_modify(|state| {
            // Update provider metrics
            let metrics = state.provider_status
                .entry(provider_id.to_string())
                .or_insert_with(ProviderMetrics::default);
            metrics.latency_ms = Some(latency_ms);
            metrics.requests_success += 1;
            
            // Update global
            state.global_stats.requests_total += 1;
            state.global_stats.requests_success += 1;
            
            // Moving average latency
            let n = state.global_stats.requests_total as f64;
            state.global_stats.avg_latency_ms = 
                (state.global_stats.avg_latency_ms * (n - 1.0) + latency_ms as f64) / n;
            
            // Add to log buffer
            state.log_buffer.push_back(LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                message: format!("Request to {} succeeded in {}ms", provider_id, latency_ms),
                provider_id: Some(provider_id.to_string()),
            });
        });
    }
}
```

### TuiAction Handler

```rust
async fn tui_action_processor(
    mut rx: mpsc::Receiver<TuiAction>,
    account_repo: Arc<dyn AccountRepository>,
    router: Arc<LlmRouter<impl AccountRepository>>,
    state_tx: watch::Sender<TuiState>,
) {
    while let Some(action) = rx.recv().await {
        match action {
            TuiAction::AddAccount { provider_id, api_key } => {
                // Persist to repository
                let account = Account::new(provider_id.clone(), api_key);
                if let Err(e) = account_repo.save(account).await {
                    // Log error
                    continue;
                }
                // Reload router
                router.reload_providers().await;
                // Send confirmation log
                let _ = state_tx.send_modify(|s| {
                    s.log_buffer.push_back(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Info,
                        message: format!("Account {} added successfully", provider_id),
                        provider_id: None,
                    });
                });
            },
            TuiAction::RemoveAccount(id) => { /* similar pattern */ },
            TuiAction::Quit => break,
        }
    }
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Telemetry injection helper | Mock TuiState, verify modifications |
| Unit | TuiAction processor | Test repository operations in isolation |
| Integration | Router → TuiState flow | Full flow with mocked provider |
| Manual | TUI with live router | `./llm-router --features tui` interactive test |

### Test Cases

1. `test_telemetry_injection_on_success` — Verify state updates after provider response
2. `test_telemetry_injection_on_failure` — Verify error state updates  
3. `test_add_account_triggers_reload` — Verify hot-reload after add
4. `test_remove_account_triggers_reload` — Verify hot-reload after remove
5. `test_signal_handler_clean_exit` — Verify TUI cleanup on SIGINT

## Migration / Rollout

No migration required. This change:
- Is additive (existing non-TUI flow unchanged)
- Uses feature gating for compilation safety
- TUI degrades gracefully if telemetry channel closed

## Open Questions

- [ ] Should provider configs also be in RwLock for hot-reload, or just the account list?
- [ ] What's the max log buffer size for production? (Currently 100, may need tuning)
- [ ] Add circuit breaker status to provider metrics in telemetry?