# Changelog

All notable changes to flightrec will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Maintained by release-plz.** Entries below [Unreleased] are generated
> automatically from Conventional Commits when a release is cut. Do not edit
> sections below [Unreleased] by hand — amend commits instead.

## [Unreleased]

## [0.2.1](https://github.com/zakelfassi/flightrec/compare/v0.2.0...v0.2.1) - 2026-06-15

### Other

- regenerate release workflow with cargo-dist

## [0.1.0] — 2026-04-01

Initial public release. The project was originally developed under the name
`agentscope` before being renamed to `flightrec` to better reflect its
instrument-grade observability focus.

### Added

- Content-addressable blob store (`~/.flightrec/objects/`)
- Snapshot engine with configurable watch roots and glob filters
- Diff engine: added / removed / modified / renamed (best-effort) change types
- Unified line diffs for text files
- Daemon loop with configurable polling interval (`watch` command)
- On-demand snapshot (`watch --once`)
- LLM narrative reporter with Anthropic, OpenAI, OpenAI-compatible, and Ollama adapters
- Ratatui TUI with timeline, diff-detail, and file-diff views (`tui` command)
- `init` command writing a starter `config.toml`
- `diff` and `replay` commands
- `FLIGHTREC_HOME` environment variable overrides storage and config location
- CI gate: `cargo fmt --all --check` → `cargo clippy --all-targets -D warnings` → `cargo test` → `cargo build --release`

[Unreleased]: https://github.com/zakelfassi/flightrec/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/zakelfassi/flightrec/releases/tag/v0.1.0
