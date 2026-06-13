# agent-session-audit

Catch exactly what your agent changed — tight file filter (`.md`, `.toml`,
`.json` only), 10-second poll interval.

**Why**: Coding agents edit config and document files. This records only those
file types at a short interval so no action slips through between snapshots.

## Run it

```sh
cp config.toml ~/.flightrec/config.toml
# Edit watch.roots to include your project and agent config dirs

flightrec watch &
FLIGHTREC_PID=$!

# ... run your agent session ...

kill $FLIGHTREC_PID
flightrec replay
```

Expected output:

```
[2026-06-12T14:00:10.001+00:00] diff-20260612T140010-001 — 2 changes
  [Modified] ~/projects/my-app/README.md
  [Added]    ~/projects/agent-config/session-notes.md

[2026-06-12T14:00:20.002+00:00] diff-20260612T140020-002 — 1 changes
  [Modified] ~/projects/my-app/Cargo.toml
```

## Interactive view

```sh
flightrec tui   # Timeline → DiffDetail → FileDiff, vim keys
```
