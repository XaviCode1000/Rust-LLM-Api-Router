#!/bin/bash
# Development workflow for Rust-LLM-Api-Router
# Optimized for Haswell 4C/8GB HDD (2025-26 stack)

set -e

echo "🚀 Starting development server with watch mode..."
echo "   - Clippy with warnings as errors"
echo "   - Nextest test runner (4x faster)"
echo "   - Test threads: 2 (optimized for 4C CPU)"
echo ""

# Start cargo-watch with clippy and nextest
cargo watch \
  -x "clippy -- -D warnings" \
  -x "nextest run --test-threads 2" \
  -d 500 \
  --clear
