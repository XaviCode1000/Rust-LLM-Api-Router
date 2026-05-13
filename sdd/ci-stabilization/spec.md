# Spec: CI Pipeline Stabilization

## Requirements

### REQ-1: Security Advisory Resolution (P1 — CRITICAL)

`cargo deny check advisories` SHALL pass without errors.

- `rustls-webpki` SHALL be updated to >=0.103.13 to resolve `RUSTSEC-2026-0104`
- Stale advisory ignores in `deny.toml` SHALL be removed (4 entries that no longer match any crate)
- `Cargo.lock` SHALL be updated accordingly

### REQ-2: Test Suite Green (P1 — CRITICAL)

ALL tests SHALL pass with `cargo nextest run --workspace`.

- `test_cli_add_provider_url_validation` SHALL be corrected to expect an error when `base_url` is empty (aligning with the validation at `provider.rs:207`)
- No new tests required — only fix the broken assertion

### REQ-3: Doctest Compilation (P2 — HIGH)

ALL doctests SHALL compile and pass with `cargo test --doc`.

- 12 failing doctests have missing imports (`ProviderRepository`, other types)
- Doctests in `presentation/mod.rs`, `failover.rs`, `auth/mod.rs`, `persistence/mod.rs`, `provider_list.rs` need import fixes
- Complex doctests that require full app setup MAY be marked `ignore` with justification

### REQ-4: Coverage Job Green (P3 — MEDIUM)

Coverage job SHALL pass. This is derived from tests passing (REQ-2) — if tests pass, coverage passes.

### REQ-5: Full CI Green (P1 — CRITICAL)

GitHub Actions CI run on `main` SHALL show ALL JOBS as `success`.

---

## Scenarios

### S-1: Security Advisory Resolved

**Given** `deny.toml` has `RUSTSEC-2026-0104` NOT in the ignore list
**And** `Cargo.lock` has `rustls-webpki >=0.103.13`
**When** `cargo deny check advisories` runs
**Then** exits with code 0

### S-2: Empty URL Test Fixed

**Given** `cmd_add_provider` rejects empty `base_url` (provider.rs:207)
**When** `test_cli_add_provider_url_validation` runs with empty URL
**Then** asserts `result.is_err()` instead of `result.is_ok()`

### S-3: Doctest Compiles

**Given** a doctest in `presentation/mod.rs` references `ProviderRepository`
**When** `cargo test --doc` runs
**Then** the doctest compiles because the necessary `use` statement is in the `no_run`/`ignore` block or the code example is marked `ignore`

### S-4: Full Pipeline Green

**Given** all individual fixes are applied
**When** CI runs on `main`
**Then** Lint ✅, Security ✅, Test ✅, Docs ✅, Coverage ✅, Live Tests ✅
