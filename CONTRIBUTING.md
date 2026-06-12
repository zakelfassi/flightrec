# Contributing to flightrec

Thank you for your interest in contributing. This document explains the development workflow, conventions, and guardrails for the project.

## Toolchain

flightrec pins to the **stable** Rust toolchain. The minimum supported version is tracked in `Cargo.toml` as `rust-version`. Install with:

```
rustup toolchain install stable
rustup default stable
```

Do not submit PRs that require nightly features.

## Development gate

Every PR must pass the full gate locally before review:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

This matches the CI matrix in `.github/workflows/ci.yml`. Green locally = green in CI.

## Commit conventions

flightrec uses **Conventional Commits** (https://www.conventionalcommits.org/). This is not stylistic — it is required because `release-plz` derives the version bump and CHANGELOG entry from the commit type:

| Type | Triggers |
|------|----------|
| `feat:` | minor version bump |
| `fix:` | patch version bump |
| `feat!:` / `fix!:` / any `!:` | major version bump |
| `docs:`, `chore:`, `refactor:`, `test:`, `ci:` | no release |

If your change is user-visible and belongs in the CHANGELOG, pick `feat` or `fix`. If it is internal cleanup, pick `chore`. When in doubt, `chore` is safer — it will not accidentally cut a release.

Example commit messages:

```
feat(snapshot): add --ignore-symlinks flag
fix(diff): handle zero-byte files without panic
docs: add VHS regen instructions to CONTRIBUTING
chore(deps): update ratatui to 0.30
```

## Design-system tokens

If you are changing TUI colors, SVG assets, or any visual output, consult `assets/BRAND.md` first. **Signal colors (`#1A9E55` added, `#C7860A` modified, `#D43B3B` removed) are reserved for change-state semantics.** Do not repurpose them for decoration, emphasis, or branding. Use the neutral palette for anything that is not a change indicator.

## Demo GIF regen (VHS)

If your change affects terminal output, regenerate the demo GIF:

```bash
brew install vhs                        # macOS
# or: go install github.com/charmbracelet/vhs@latest
vhs assets/demo.tape                    # outputs assets/demo.gif
```

Target: ≤2.5 MB at 1200×675. Check file size before committing. The tape file sets up a temporary `FLIGHTREC_HOME` in a `Hidden` block — do not show setup noise in the recording.

## Skills (SkDD)

The `skills/` directory contains reusable agent skills following the [agentskills.io](https://agentskills.io) specification. The registry is in `.skills-registry.md`. If you add or update a skill, update the registry entry (`last-used`, `usage-count`, or add a new row). Skill `SKILL.md` frontmatter must contain only `name` and `description` (plus an optional `metadata` block). Do not add nonstandard keys.

## Pull requests

Open a PR against `master`. The PR description should follow the template in `.github/PULL_REQUEST_TEMPLATE.md`. One feature or fix per PR; stacked PRs are welcome. Link the relevant issue if one exists.

## Code of conduct

All contributors are expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
