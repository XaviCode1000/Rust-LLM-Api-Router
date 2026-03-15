#!/bin/bash
# Generate coverage report with llvm-cov (10x faster than tarpaulin)
# Optimized for Haswell 4C/8GB HDD

set -e

echo "📊 Generating coverage report with llvm-cov..."
echo "   - This is ~10x faster than cargo-tarpaulin"
echo "   - Output: coverage-llvm/"
echo ""

# Generate HTML coverage report
cargo llvm-cov \
  --html \
  --output-dir coverage-llvm \
  --test-threads 2 \
  --open

echo ""
echo "✅ Coverage report generated!"
echo "   Open: coverage-llvm/index.html"
echo ""

# Show summary
cargo llvm-cov --summary-only 2>&1 | grep -E "(TOTAL|Region)"
