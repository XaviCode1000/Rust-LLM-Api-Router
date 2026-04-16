# Proposal: Sprint 3 - Interactive Forms & Command Control

## Intent

Enable interactive control of the LLM Router system through the TUI. Currently the TUI is read-only (passive dashboard). The new requirement adds input handling for account management and provider toggling. This addresses the threat analysis findings: API key exposure, UI blocking, focus ambiguity, and malformed data injection.

## Scope

### In Scope
- TuiAction enum with AddAccount, RemoveAccount, ToggleProvider, Quit variants
- InputMode state machine (Normal → Editing → Processing → Normal)
- Form widget with masked API key input (popup pattern)
- Processing spinner during async validation
- mpsc channel (32 buffer) for TuiAction communication

### Out of Scope
- OAuth/PKCE flow UI (covered in secure-keyring change)
- Provider configuration UI beyond toggle (add/edit)
- Persistent form state (stateless after submission)
- Multiple concurrent forms

## Capabilities

### New Capabilities
- `tui-interactive-forms`: Interactive account management through TUI with masked API keys and async validation

### Modified Capabilities
- `tui-dashboard`: Extends from read-only to interactive with input handling

## Approach

Add InputMode enum to state.rs with Normal/Editing/Processing states. Create TuiAction enum in mod.rs with mpsc channel. Extend engine.rs to handle key events for mode transitions, render popup forms with Clear overlay, and display spinner during Processing. Security: API key rendered as `*` in UI, real value only in edit buffer, cleared after mpsc send.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/presentation/tui/mod.rs` | Modified | Add TuiAction enum, mpsc channel |
| `src/presentation/tui/state.rs` | Modified | Add InputMode enum, form state fields |
| `src/presentation/tui/engine.rs` | Modified | Add popup rendering, input handling, spinner |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Race condition on form state | Low | Single-threaded TUI, Processing blocks input |
| API key log exposure | Low | Never log API key, only errors with provider_id |
| Input validation blocking | Medium | Processing state disables submit, debounce |

## Rollback Plan

Revert to read-only dashboard by removing InputMode and form handling logic. TuiAction channel unused but harmless (no producer yet). Restore engine.rs to original draw_dashboard-only pattern.

## Dependencies

- None (pure TUI changes)

## Success Criteria

- [ ] TuiAction channel compiles and spawns at startup
- [ ] InputMode transitions correctly on key events
- [ ] API key never appears in rendered UI (only `*`)
- [ ] Spinner animates during Processing state
- [ ] Error states display in log panel