# justfile — Rust-LLM-Api-Router
# Complementa a bacon (inner loop). Esto es para tareas manuales (outer loop).

# Stack: Rust 1.93 · Axum · reqwest · tokio

# -- Verificación --

default: check

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings

check-fast:
    cargo check

# -- Tests --

test:
    cargo nextest run --test-threads 2

test-all:
    cargo nextest run --test-threads 2 --all-features

# -- Auditoría --

audit:
    cargo audit
    cargo deny check

audit-all:
    cargo audit
    cargo deny check licenses
    cargo deny check advisories

# -- Coverage --

cov:
    cargo llvm-cov nextest --html --output-dir coverage-llvm

cov-summary:
    cargo llvm-cov nextest --summary-only

# -- Format --

fmt:
    cargo fmt

# -- Build --

build:
    cargo build

build-release:
    cargo build --release

# -- CI --

test-ci:
    cargo nextest run --profile ci

# -- Maintenance --

sccache-stats:
    sccache --show-stats

sccache-clear:
    sccache --zero-stats

# -- Setup --

setup:
    @echo "Verificando herramientas..."
    @which cargo-nextest || (echo "Falta: cargo install cargo-nextest"; exit 1)
    @which cargo-llvm-cov || (echo "Falta: cargo install cargo-llvm-cov"; exit 1)
    @which just || (echo "Falta: cargo install just"; exit 1)
    @echo "✓ Todas las herramientas instaladas"