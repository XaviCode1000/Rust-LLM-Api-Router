# Development Workflow — Rust-LLM-Api-Router

## 🎉 Latest Achievements

**Coverage:** **80.35%** (32% → 80.35%, +48.33%)  
**Tests:** **492 passing** (104 → 492, +388 tests)  
**Status:** ✅ **All tests passing, 0 failing**

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

---

## 📦 Stack Óptimo 2025-26

| Herramienta | Versión | Propósito |
|-------------|---------|-----------|
| **Rust** | 1.93.0 | Latest stable |
| **cargo-nextest** | 0.9.130 | Test runner (4x faster) |
| **cargo-llvm-cov** | latest | Cobertura nativa LLVM (10x faster) |
| **sccache** | 0.14.0 | Cache de compilación (6x faster) |
| **cargo-watch** | 8.5.3 | Auto-recompilar en cambios |

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
# Format + lint + test
cargo fmt
cargo clippy -D warnings
cargo nextest run --test-threads 2
```

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

## 📚 Resources

- [cargo-nextest docs](https://nexte.st/)
- [cargo-llvm-cov docs](https://github.com/taiki-e/cargo-llvm-cov)
- [sccache docs](https://github.com/mozilla/sccache)
- [Rust 2025-26 Best Practices](https://rust-lang.github.io/api-guidelines/)

---

**Last updated**: 2026-03-29  
**Rust version**: 1.93.0  
**Stack version**: 2025-26 optimal
