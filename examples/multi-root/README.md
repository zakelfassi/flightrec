# multi-root

Watch several directories in one daemon — project, agent config, and dotfiles
in a single audit trail.

**Why**: When an agent touches multiple directories, one flightrec instance
captures the full picture. One diff event spans all roots.

## Run it

```sh
cp config.toml ~/.flightrec/config.toml
# Edit watch.roots — add as many directories as needed

flightrec watch --once   # baseline
# let your agent run
flightrec watch --once   # record
```

Expected output when changes span roots:

```
[2026-06-12T09:30:00.123+00:00] snapshot 20260612T093000-123 — 47 files → ~/.flightrec/snapshots/20260612T093000-123.json
  3 changes:
    ~ ~/projects/my-app/src/main.rs
    + ~/projects/agent-config/prompts/system.md
    ~ ~/.dotfiles/zshrc
```

## Replay

```
[2026-06-12T09:30:00.123+00:00] diff-20260612T093000-123 — 3 changes
  [Modified]  ~/projects/my-app/src/main.rs
  [Added]     ~/projects/agent-config/prompts/system.md
  [Modified]  ~/.dotfiles/zshrc
```

Non-existent roots are silently skipped — safe to include paths that may not
exist on every machine.
