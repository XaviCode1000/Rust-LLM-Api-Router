# Tasks: Docker & Containerization (Issue #15)

## Implementation Tasks

### Phase 1: Docker Files
- [ ] **T1: Create `Dockerfile`** — Multi-stage build (builder + runtime), non-root user, healthcheck, volume
- [ ] **T2: Create `.dockerignore`** — Exclude target/, .git/, docs/, sdd/, coverage/, etc.
- [ ] **T3: Create `docker-compose.yml`** — Local dev setup with volume, env vars, healthcheck

### Phase 2: CI/CD
- [ ] **T4: Create `.github/workflows/docker.yml`** — Build and push to GHCR on main/tags

### Phase 3: Documentation
- [ ] **T5: Create `docs/deployment.md`** — How to build, run, configure, deploy

### Phase 4: Verification
- [ ] **T6: Verify Dockerfile syntax** — `docker build --no-cache .` (if Docker available)
- [ ] **T7: Verify docker-compose syntax** — `docker compose config`
- [ ] **T8: Verify all tests pass** — `cargo nextest run --test-threads 2`
- [ ] **T9: Format and lint** — `cargo fmt --check` and `cargo clippy -- -D warnings`

## Dependencies

```
T1 → T2 → T3
T4 (after T1)
T5 (after T1-T3)
T6-T9 (after T1-T5)
```

## Notes

- No Rust code changes — only Docker/CI/CD files
- Binary is already optimized with LTO fat + strip in Cargo.toml
- Health endpoint already exists at GET /health
- Data directory configurable via XDG_CONFIG_HOME
