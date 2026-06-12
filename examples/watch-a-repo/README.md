# watch-a-repo

Record every source file change in a single project directory.

**Why**: The quickest way to start — point flightrec at one project and get a
complete, replayable diff history from the first snapshot.

## Run it

```sh
cp config.toml ~/.flightrec/config.toml
# Edit watch.roots to point at your project

flightrec watch --once   # baseline
# edit a file
flightrec watch --once   # record the diff
```

Expected output:

```
[2026-06-12T09:15:30.456+00:00] snapshot 20260612T091530-456 — 12 files → ~/.flightrec/snapshots/20260612T091530-456.json
  1 changes:
    ~ ~/projects/my-app/README.md
```

## Replay

```sh
flightrec replay
```

```
[2026-06-12T09:15:30.456+00:00] diff-20260612T091530-456 — 1 changes
  [Modified] ~/projects/my-app/README.md
```

Continuous watch: `flightrec watch` — polls every 30 s (`daemon.interval_seconds`).
