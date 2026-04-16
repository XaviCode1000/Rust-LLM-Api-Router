# AGENTS.md — Rust-LLM-Api-Router

> Simplified. See `.atl/skill-registry.md` for full context.

## What

LLM API Router. High-performance proxy across 34 providers with failover, cascading routing, and intelligent model selection.

## Commands

```bash
just check          # fmt --check + clippy -D warnings
just test           # cargo nextest run --test-threads 2
just audit          # cargo audit + cargo deny check
just cov            # cargo llvm-cov nextest --html
just fmt            # cargo fmt
just build-release  # cargo build --release
```

## Non-Standard Tooling

- **Testing**: `cargo-nextest` (not cargo test), `cargo-llvm-cov` (not tarpaulin)
- **Task orchestration**: `just` (not raw scripts)
- **Build cache**: `sccache`

## Do

- Use `thiserror` for domain errors, `anyhow` for application errors
- Use `tokio::sync::Mutex`, NOT `std::sync::Mutex` in async contexts
- Use `Arc<TokioMutex<T>>` for shared state
- Keep Domain layer pure (no external deps except serde)
- Use traits for dependency injection

## Don't

- **NEVER** use `unwrap()`/`expect()` in production — use `?` or match
- **NEVER** hold locks across `.await` — scope locks tightly
- **NEVER** use `format!()` in hot paths — use write! or format!
- **NEVER** commit secrets — use environment variables
- **NEVER** use `cargo test` (use nextest) or `cargo tarpaulin` (use llvm-cov)

## Progressive Disclosure

All project-specific knowledge is in skills, loaded on demand:

| Task | Skill |
|------|-------|
| Tests (522 symbols, 60 files) | `tests` |
| Services (136 symbols, 14 files) | `services` |
| Auth (31 symbols, 5 files) | `auth` |
| GitNexus exploration | auto-loaded |
| GitNexus impact analysis | auto-loaded |
| GitNexus debugging | auto-loaded |

Full registry: `.atl/skill-registry.md`

## References

- Architecture: `docs/architecture.md`
- Routing: `docs/routing.md`
- Testing guide: `docs/TESTING_GUIDE.md`
- CLI reference: `docs/cli.md`

## GitNexus

This project is indexed. Run `gitnexus_query` for execution flows, `gitnexus_context` for symbol details. See `.claude/skills/gitnexus/` for GitNexus skills.

---

**Last Updated**: April 2026
**Quick Ref**: Run `just` for all commands