#!/bin/bash
# run.sh - agentscope-build skill entrypoint

export PATH="$HOME/.cargo/bin:$PATH"

# Format check
echo "Step 1: Checking formatting..."
if ! cargo fmt -- --check; then
    echo "Formatting failed."
    exit 1
fi

# Clippy lint
echo "Step 2: Linting with clippy..."
if ! cargo clippy -- -D warnings; then
    echo "Clippy failed."
    exit 2
fi

# Tests
echo "Step 3: Running tests..."
if ! cargo test; then
    echo "Tests failed."
    exit 3
fi

# Build
echo "Step 4: Building release binary..."
if ! cargo build --release; then
    echo "Build failed."
    exit 4
fi

echo "Success! Build complete."
exit 0
