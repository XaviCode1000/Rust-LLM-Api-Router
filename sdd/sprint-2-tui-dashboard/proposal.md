# Proposal: Sprint 2 - TUI Dashboard Widgets & Event Loop

## Intent

Implement the Terminal User Interface dashboard with real-time metrics visualization using Ratatui. This delivers Sprint 2 of the TUI project, following Sprint 1's TuiState and watch channel infrastructure. Currently, the TUI infrastructure exists but has no render loop, inefficient memory cloning, and missing panic handling.

## Scope

### In Scope
- Memory optimization via Arc-wrapped heavy fields in TuiState
- TuiEngine with async event loop using tokio::select!
- Pure widget functions (draw_header, draw_provider_table, draw_logs with sparkline)
- Panic hook for terminal cleanup
- 3-area responsive layout (Header 10%, Providers 60%, Logs 30%)

### Out of Scope
- Interactive provider management (future sprint)
- Historical metrics storage (separate persistence)
- Configuration UI (future sprint)

## Capabilities

### New Capabilities
- `tui-dashboard`: Real-time metrics visualization in terminal with provider health, latency sparklines, and log streaming

### Modified Capabilities
- None (new capability)

## Approach

**Memory Optimization**: Wrap `provider_status` HashMap and `log_buffer` in `Arc<Mutex<...>>` inside TuiState. Watch channel clones lightweight Arc pointers instead of full data.

**TuiEngine Architecture**:
1. Async task spawns to `spawn_blocking` for RATTerminal operations
2. Event loop uses `tokio::select!` with 3 branches: state changes, timer tick, input poll
3. Dirty-flag pattern: only render when state.dirty == true

**Widget Design**:
- Pure, monomorphic functions taking `&TuiState`
- Tactical colors: Cyan (healthy), Red+BOLD (circuit open), Yellow (degraded)
- TableState for scrolling, sorted by provider_id

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/presentation/tui/state.rs` | Modified | Arc-wrapped heavy fields |
| `src/presentation/tui/engine.rs` | New | TuiEngine, event loop, widgets |
| `src/presentation/tui/mod.rs` | Modified | Export new engine |
| `src/main.rs` | Modified | Panic hook at startup |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Terminal corruption on panic | High | panic::set_hook with disable_raw_mode |
| Input blocking on Windows | Medium | Use crossterm async events API |
| HashMap disorder causes flicker | High | Sort by provider_id before render |
| Watch channel clone overhead | Medium | Arc<HashMap> reduces to pointer clone |

## Rollback Plan

1. Remove `--features tui` from cargo run command
2. Revert state.rs: replace Arc-wrapped fields with direct HashMap/VecDeque
3. Comment out TuiEngine::run() call in main.rs
4. Original watch channel still functional (state is cloned but not rendered)

## Dependencies

- ratatui 0.30 (already in Cargo.toml, feature-gated)
- crossterm 0.28 (already in Cargo.toml)
- tokio (already dependency)

## Success Criteria

- [ ] TUI renders without crash on `cargo run --features tui`
- [ ] Provider table sorts consistently (no jumping)
- [ ] Panic (Ctrl+C) cleans up terminal properly
- [ ] Memory: watch channel clone copies < 1KB not MB
- [ ] Frame rate: ~60fps (16ms tick) smooth scrolling