#!/usr/bin/env bash
# Setup Git hooks for Rust formatting enforcement
# Copies hooks from githooks/ to .git/hooks/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITHOOKS_DIR="$PROJECT_ROOT/githooks"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

if [ ! -d "$GITHOOKS_DIR" ]; then
    echo "Error: githooks directory not found at $GITHOOKS_DIR"
    exit 1
fi

echo "Installing Git hooks from $GITHOOKS_DIR to $GIT_HOOKS_DIR..."

# Backup existing hooks if they exist
for hook in pre-commit pre-push; do
    if [ -f "$GIT_HOOKS_DIR/$hook" ]; then
        echo "Backing up existing $hook hook to $hook.backup"
        cp "$GIT_HOOKS_DIR/$hook" "$GIT_HOOKS_DIR/$hook.backup"
    fi
done

# Copy hooks
cp "$GITHOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/"
cp "$GITHOOKS_DIR/pre-push" "$GIT_HOOKS_DIR/"

# Make executable
chmod +x "$GIT_HOOKS_DIR/pre-commit"
chmod +x "$GIT_HOOKS_DIR/pre-push"

echo "Git hooks installed successfully!"
echo "- pre-commit: runs cargo fmt automatically before each commit"
echo "- pre-push: verifies formatting before push"
echo ""
echo "To skip hooks temporarily, use: git commit --no-verify"