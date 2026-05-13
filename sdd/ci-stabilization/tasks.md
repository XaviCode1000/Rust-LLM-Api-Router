# Tasks: CI Pipeline Stabilization

## Execution Order

1. **Task 1** (Security) → Task 2 (Tests) → Task 3 (Doctests) → Task 4 (Verify CI)

---

## Task 1: Resolve Security Advisory (P1 — CRITICAL)

### 1a: Update rustls-webpki

- [ ] Run `cargo update -p rustls-webpki` to upgrade from v0.103.12 to >=0.103.13
- [ ] Verify: `cargo check` still passes
- [ ] Verify: `cargo deny check advisories` passes (RUSTSEC-2026-0104 resolved)

### 1b: Clean stale deny.toml entries

- [ ] Remove from `deny.toml`:
  - `RUSTSEC-2026-0049` — no longer matches any crate
  - `RUSTSEC-2026-0098` — no longer matches any crate
  - `RUSTSEC-2026-0099` — no longer matches any crate
  - `RUSTSEC-2026-0066` — no longer matches any crate (if confirmed)
- [ ] Keep `RUSTSEC-2024-0388` and `RUSTSEC-2024-0384` (unmaintained deps — still relevant)
- [ ] Verify: `cargo deny check advisories` passes with clean config

**Files**: `Cargo.lock`, `deny.toml`

---

## Task 2: Fix Failing Test (P1 — CRITICAL)

**File**: `tests/provider_commands_integration_tests.rs:75-80`

### Current (broken):
```rust
// Empty URL - should add (no validation)
let args2 = create_add_args("empty-url", "Empty URL Provider", "", Some("sk-key"));
let result2 = cmd_add_provider(args2, &repo).await;
assert!(result2.is_ok());
```

### Fix:
```rust
// Empty URL - rejected by validation (base_url required)
let args2 = create_add_args("empty-url", "Empty URL Provider", "", Some("sk-key"));
let result2 = cmd_add_provider(args2, &repo).await;
assert!(result2.is_err(), "Empty URL should be rejected");
```

- [ ] Update assertion from `is_ok()` to `is_err()`
- [ ] Update comment to match new behavior
- [ ] Verify: `cargo nextest run test_cli_add_provider_url_validation` passes
- [ ] Verify: `cargo nextest run --workspace` passes (all 751 tests)

---

## Task 3: Fix Doctests (P2 — HIGH)

12 doctests failing across these files:
- `src/presentation/mod.rs` (line 38) — `ProviderRepository` not found
- `src/app/services/failover.rs` (lines 45, 93, 151) — missing imports
- `src/infrastructure/auth/mod.rs` (lines 44, 70, 99, 122) — missing imports
- `src/infrastructure/persistence/mod.rs` (line 23) — missing imports
- `src/presentation/cli/commands/provider_list.rs` (lines 23, 56, 108) — missing imports

### Approach:
For each failing doctest:
1. Read the doctest
2. Add required `use` statements inside the doctest block
3. If the doctest requires full app setup (async runtime, repositories), mark as `# #[ignore]` with comment explaining why
4. Verify: `cargo test --doc` passes

- [ ] Fix `presentation/mod.rs` doctest
- [ ] Fix `failover.rs` doctests (3)
- [ ] Fix `auth/mod.rs` doctests (4)
- [ ] Fix `persistence/mod.rs` doctest
- [ ] Fix `provider_list.rs` doctests (3)
- [ ] Verify: `cargo test --doc` passes

---

## Task 4: Verify Full CI Green

- [ ] `cargo fmt --check` — PASS
- [ ] `cargo clippy -- -D warnings` — PASS
- [ ] `cargo doc --no-deps` — PASS
- [ ] `cargo deny check advisories` — PASS
- [ ] `cargo deny check licenses` — PASS
- [ ] `cargo deny check bans` — PASS
- [ ] `cargo nextest run --workspace` — ALL PASS
- [ ] `cargo test --doc` — ALL PASS
- [ ] Push to `main`
- [ ] Verify GitHub Actions ALL JOBS GREEN
