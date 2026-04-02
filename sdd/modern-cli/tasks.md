# Tasks: Modern Interactive CLI Experience (Issue #19)

## Implementation Tasks

### Phase 1: Foundation (Utilities)
- [ ] **T1: Add dependencies** — Add `owo-colors`, `comfy-table`, `inquire`, `indicatif`, `is-terminal` to `Cargo.toml`
- [ ] **T2: Create `tty.rs`** — TTY detection + `NO_COLOR` support with atomic state
- [ ] **T3: Create `output.rs`** — Colored output utilities (success, error, warning, info, dim, bold)
- [ ] **T4: Create `spinner.rs`** — `CliSpinner` wrapper around `indicatif` with TTY awareness
- [ ] **T5: Create `table.rs`** — `provider_table()` and `account_table()` using `comfy-table`
- [ ] **T6: Create `prompt.rs`** — Interactive prompts (confirm, text, secret, select) with TTY fallback
- [ ] **T7: Update `input.rs`** — Replace `stdin().read_line()` with `inquire` masked input

### Phase 2: Command Refactoring
- [ ] **T8: Update `mod.rs`** — Add TTY init, update imports, add `--no-color` flag to Cli struct
- [ ] **T9: Refactor `provider.rs`** — Colored output, tables for list, confirmations for remove, spinner for validate, rich help examples
- [ ] **T10: Refactor `account.rs`** — Colored output, tables for list, confirmations for remove, spinner for validate
- [ ] **T11: Refactor `login.rs`** — Spinner during OAuth flow, colored success/error messages
- [ ] **T12: Refactor `logout.rs`** — Confirmation for `--all`, colored output
- [ ] **T13: Refactor `auth.rs`** — Colored output wrapper

### Phase 3: Verification
- [ ] **T14: Verify compilation** — `cargo check` passes with zero errors
- [ ] **T15: Run tests** — `cargo nextest run --test-threads 2` all tests pass
- [ ] **T16: Format and lint** — `cargo fmt --check` and `cargo clippy -- -D warnings` clean
- [ ] **T17: Manual testing** — Test all CLI commands interactively (add, list, remove, validate)
- [ ] **T18: Update docs** — Update `docs/cli.md` with new output examples

## Dependencies

```
T1 → T2 → T3 → T4 → T5 → T6 → T7
T8 (after T2-T7)
T9 (after T8, T3-T6)
T10 (after T8, T3-T6)
T11 (after T8, T3-T4)
T12 (after T8, T3, T6)
T13 (after T8, T3)
T14-T18 (after T9-T13)
```

## Notes

- All changes are in `src/presentation/cli/` — no domain or infrastructure changes
- Backward compatibility: same flags, same arguments, same behavior
- New features are additive: colors, tables, confirmations, spinners
- Graceful degradation for non-TTY environments is mandatory
- `inquire` calls must use `spawn_blocking` in async context
