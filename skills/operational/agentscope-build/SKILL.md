---
name: agentscope-build
description: Build and test the agentscope daemon (fmt -> clippy -> test -> build)
triggers: [manual, on-push]
---

# agentscope-build

This skill handles the full build and verification pipeline for the agentscope daemon.

## Inputs
- **codebase**: The current working directory (Rust source)

## Outputs
- **artifact**: `target/release/agentscope` binary

## Steps
1. **Format**: Check code formatting (`cargo fmt --check`)
2. **Lint**: Run clippy (`cargo clippy -- -D warnings`)
3. **Test**: Run unit tests (`cargo test`)
4. **Build**: Compile release binary (`cargo build --release`)

## Usage
Run directly:
```bash
./run.sh
```
