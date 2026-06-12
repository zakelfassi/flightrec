## Summary

<!-- 1-3 bullets describing what this PR changes and why. -->

-

## Linked issue

<!-- Closes #NNN / Refs #NNN — or "none" if standalone. -->

## Conventional commit title check

<!-- The PR title (and squash commit) must follow Conventional Commits:
     feat: / fix: / docs: / chore: / refactor: / test: / ci: / feat!: / fix!:
     release-plz uses this to derive the version bump and CHANGELOG entry. -->

- [ ] PR title follows the Conventional Commits format

## Gate checklist

Run locally before requesting review:

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --all-targets` passes
- [ ] `cargo build --release` passes

## User-facing change

- [ ] This PR changes CLI flags, output format, config schema, or file layout in a way that users will notice

<!-- If checked, describe the change in the Summary above so it can be
     included verbatim in the CHANGELOG entry. -->
