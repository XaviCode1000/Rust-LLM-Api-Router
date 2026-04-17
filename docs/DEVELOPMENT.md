# Development Workflow — Rust-LLM-Api-Router

## 🎉 Latest Achievements

**Coverage:** **80.35%** (32% → 80.35%, +48.33%)
**Tests:** **~680+ passing** (104 → ~680+, including 19 proptest, 5 snapshot, 3 live contract)
**MSRV:** 1.80 (was 1.75)
**Clippy:** ✅ Clean (0 errors, 0 warnings)
**Secret Scanning:** ✅ gitleaks CI enabled
**Status:** ✅ All core tests passing (4 pre-existing failures unrelated to audit fixes)

### Audit Remediation (Issue #30)

- ✅ Removed hardcoded Cloudflare credential
- ✅ Fixed `block_on` deadlock → async `.await`
- ✅ Fixed Clean Architecture violations (domain layer purged of framework imports)
- ✅ Error source chain preserved with `#[from]`
- ✅ Moved `IntoResponse` impls to presentation layer
- ✅ Fixed 10 clippy errors
- ✅ MSRV updated to 1.80
- ✅ Hot path optimization: typed `ProviderChatRequest` replaces `serde_json::json!()`
- ✅ 19 property-based tests for routing logic
- ✅ gitleaks CI workflow added

See [`docs/TESTING_JOURNEY.md`](docs/TESTING_JOURNEY.md) for the complete story.

---

## 🚀 Quick Start

```bash
# Install tools (one-time, already done ✅)
cargo install cargo-nextest cargo-llvm-cov sccache cargo-watch cargo-binstall

# Start development server
./scripts/dev.sh

# Generate coverage report
./scripts/coverage.sh
```

### Docker Development (Issue #15)

```bash
# Build Docker image locally
docker build -t llm-router:dev .

# Run with hot-reload (mount source)
docker run -it --rm \
  -v $(pwd):/app \
  -p 8080:8080 \
  llm-router:dev

# Run with docker-compose for full stack
docker compose up -d

# Pull latest from GHCR
docker pull ghcr.io/xavicode1000/rust-llm-api-router:latest
```

---

## 📦 Stack Óptimo 2025-26

| Herramienta | Versión | Propósito |
|-------------|---------|-----------|
| **Rust** | 1.93.0 | Latest stable |
| **cargo-nextest** | 0.9.130 | Test runner (4x faster) |
| **cargo-llvm-cov** | latest | Cobertura nativa LLVM (10x faster) |
| **sccache** | 0.14.0 | Cache de compilación (6x faster) |
| **cargo-watch** | 8.5.3 | Auto-recompilar en cambios |
| **Docker** | 20.10+ | Containerization |
| **docker-compose** | v2+ | Multi-container orchestration |

### New Dependencies (Issues #19, #22)

| Crate | Purpose | Issue |
|-------|---------|-------|
| `owo-colors` | Colored terminal output | #19 |
| `comfy-table` | Professional CLI tables | #19 |
| `inquire` | Interactive CLI prompts | #19 |
| `indicatif` | Progress spinners | #19 |
| `is-terminal` | TTY detection | #19 |
| `keyring` | System keyring access | #22 |
| `aes-gcm` | Encrypted file storage | #22 |
| `argon2` | Key derivation | #22 |

---

## 🛠️ Commands

### Tests

```bash
# Traditional (slow)
cargo test -- --test-threads 2

# Nextest (4x faster) ✅
cargo nextest run --test-threads 2

# Watch mode (auto-rerun) ✅
./scripts/dev.sh
```

### Cobertura

```bash
# Tarpaulin (slow, ~5min)
cargo tarpaulin --out Html

# LLVM-Cov (fast, ~30s) ✅
cargo llvm-cov --html --output-dir coverage-llvm

# With watch mode
./scripts/coverage.sh
```

### Build

```bash
# Standard build
cargo build --release

# With sccache (6x faster) ✅
sccache --show-stats  # View cache stats
```

### Linting

```bash
# Clippy with warnings as errors
cargo clippy -D warnings

# Auto-fix
cargo clippy --fix
```

### Formatting

```bash
# Check format
cargo fmt --check

# Format code
cargo fmt
```

### Git Hooks

To prevent formatting issues in CI, the repository includes Git hooks that automatically run `cargo fmt` before each commit and verify formatting before push.

**Pre-commit hook:** Runs `cargo fmt --all` automatically and stages formatted files. You don't need to remember to format code manually.

**Pre-push hook:** Runs `cargo fmt --all -- --check` to ensure all code is properly formatted before pushing to remote.

The hook scripts are stored in the `githooks/` directory (version-controlled). To install them in your local repository, run:

```bash
./scripts/setup-githooks.sh
```

This will copy the hooks to `.git/hooks/` and make them executable. If you already have custom hooks, they will be backed up with a `.backup` extension.

**Note:** If you have the `pre-commit` Python tool installed, the original hook has been backed up to `.git/hooks/pre-commit.pre-commit-bak`.

---

## 📊 Performance Comparison

| Task | Traditional | Optimized 2025-26 | Mejora |
|------|-------------|-------------------|--------|
| **Tests** | `cargo test` (31s) | `cargo nextest` (8s) | **~4x** |
| **Coverage** | `tarpaulin` (5min) | `llvm-cov` (30s) | **~10x** |
| **Build** | Clean (60s) | `sccache` (10s) | **~6x** |
| **Watch** | Manual | `cargo-watch` | **Instant** |

---

## 🔧 Configuration

### `.cargo/config.toml`

```toml
[build]
rustc-wrapper = "sccache"

[alias]
nextest = "nextest"
llvm-cov = "llvm-cov"
watch = "watch"
```

### `sccache` Stats

```bash
# Start server
sccache --start-server

# View stats
sccache --show-stats

# Zero stats
sccache --zero-stats
```

---

## 📁 Project Structure

```
Rust-LLM-Api-Router/
├── .cargo/
│   └── config.toml          # sccache + aliases
├── scripts/
│   ├── dev.sh               # Watch mode + clippy + nextest
│   └── coverage.sh          # LLVM coverage report
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Public API + re-exports
│   ├── domain/              # Entities, traits, errors
│   ├── app/                 # Services, router, execution_plan
│   ├── infrastructure/      # HTTP, persistence, auth, providers
│   ├── interfaces/          # HTTP handlers
│   └── presentation/        # Routes, state, CLI
│       └── cli/
│           ├── mod.rs       # Cli struct, dispatcher
│           ├── commands/
│           │   ├── provider.rs    # Provider CRUD
│           │   ├── account.rs     # Account CRUD
│           │   ├── auth.rs        # Login/logout
│           │   └── completions.rs # Shell completions (feature-gated)
│           └── input.rs           # Shared input helpers
├── tests/
│   ├── error_snapshots.rs   # Insta snapshot tests
│   ├── failover_integration.rs
│   ├── security_tests.rs
│   ├── mock_http_tests.rs   # Wiremock tests
│   └── failover_chaos.rs    # Turmoil chaos tests
├── coverage-llvm/           # Generated coverage report
└── docs/
    ├── cli.md               # CLI reference
    ├── architecture.md      # Architecture docs
    └── coverage-report.md   # Coverage analysis
```

---

## 🎯 Testing Workflow

### 1. Daily Development

```bash
# Terminal 1: Watch mode (auto-rerun tests)
./scripts/dev.sh

# Terminal 2: Edit code → tests auto-rerun
```

### 2. Before Commit

```bash
# Lint + test (formatting is automatic via pre-commit hook)
cargo clippy -D warnings
cargo nextest run --test-threads 2
```

**Note:** Pre-commit hook runs `cargo fmt` automatically. Pre-push hook verifies formatting before push.

### 3. Coverage Check

```bash
# Generate and open coverage report
./scripts/coverage.sh

# Target: >80% en código crítico
# Current: 80.35% ✅
```

---

## 🐛 Troubleshooting

### sccache no funciona

```bash
# Verificar servidor
sccache --show-stats

# Reiniciar
sccache --stop-server
sccache --start-server
```

### Nextest falla

```bash
# Limpiar build
cargo clean

# Reintentar
cargo nextest run --test-threads 2
```

### Cobertura no genera

```bash
# Limpiar artifacts
cargo clean

# Regenerar
cargo llvm-cov --clean --html
```

---

## 🔐 Secure Storage Setup for Development (Issue #22)

### Overview

The project supports secure API key storage via system keyrings or encrypted files.

### Configuration

```bash
# Default: auto-detect (use keyring if available)
export SECURE_STORAGE=auto

# Force encrypted file storage
export SECURE_STORAGE=encrypted

# Disable secure storage (for testing only!)
export SECURE_STORAGE=disabled
```

### Development Setup

```bash
# For local development with keyring support (Linux)
# Install libsecret for Secret Service
sudo apt install libsecret-1-dev

# Verify keyring is available
llm-router account secure-status

# Migrate existing keys to secure storage
llm-router account migrate
```

### Testing Secure Storage

```bash
# Test with encrypted file storage
SECURE_STORAGE=encrypted cargo nextest run

# Test disabled (for rapid iteration)
SECURE_STORAGE=disabled cargo nextest run
```

---

## 🖥️ CLI Testing (Issue #19)

The CLI has been enhanced with modern interactive features.

### Testing Interactive Prompts

```bash
# Test interactive account add
llm-router account add --interactive

# Test provider add
llm-router provider add --interactive

# Force interactive mode in non-TTY
llm-router --force-interactive account list
```

### Testing Colored Output

```bash
# Verify colors work
llm-router provider list

# Check for color codes in output
llm-router provider list | cat -v
```

### Testing Tables

```bash
# Verify table formatting
llm-router provider list
llm-router account list
```

### Testing TTY Detection

```bash
# Run with TTY
script -q /dev/null -c "llm-router account list"

# Run without TTY (should show simplified output)
llm-router account list | cat
```

---

## 📚 Resources

- [cargo-nextest docs](https://nexte.st/)
- [cargo-llvm-cov docs](https://github.com/taiki-e/cargo-llvm-cov)
- [sccache docs](https://github.com/mozilla/sccache)
- [Rust 2025-26 Best Practices](https://rust-lang.github.io/api-guidelines/)

---

**Last updated**: 2026-04-17 (documentation cleanup: re-indexed GitNexus)
**Rust version**: 1.80+ (MSRV)
**Stack version**: 2025-26 optimal (Tokio 1.x, Axum 0.7, Tower 0.5)
