---
name: flightrec-build
description: Build and verify flightrec (fmt --all --check → clippy --all-targets -D warnings → test → build --release). Use before opening a PR or cutting a release.
---

# flightrec-build

Full build and verification pipeline for the flightrec crate.

## Inputs

- Working directory: the flightrec repository root (Rust source + `Cargo.toml`)

## Outputs

- `target/release/flightrec` binary

## Steps

1. **Format**: `cargo fmt --all --check`
2. **Lint**: `cargo clippy --all-targets -- -D warnings`
3. **Test**: `cargo test --all-targets`
4. **Build**: `cargo build --release`

## Usage

```bash
bash skills/flightrec-build/run.sh
```

## Edge Cases

- If `rustfmt.toml` overrides edition or max_width, the fmt check reflects those settings.
- Clippy warnings promoted to errors (`-D warnings`) include `dead_code` and `unused_imports` — fix rather than `#[allow]`.
- The release build is slow on first run (no incremental cache). Use `sccache` if available.
