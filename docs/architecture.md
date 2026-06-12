# flightrec — Architecture

This document describes the **actual v0.2 implementation**. Planned items are marked 🚧 and listed in [ROADMAP.md](../ROADMAP.md).

## Pipeline

```
  watch roots
  (config.toml)
       │
       ▼
┌─────────────────┐
│  Snapshot       │  walk + filter → hash files → write blobs
│  Engine         │  → SnapshotManifest (JSON)
└────────┬────────┘
         │  new snapshot
         ▼
┌─────────────────┐
│  Diff Engine    │  compare two manifests → added/removed/modified/renamed
│                 │  → load blobs → unified diffs for text files
└────────┬────────┘
         │  DiffEvent (JSON)
         ▼
┌─────────────────┐    ┌────────────────────┐
│  LLM Reporter   │───▶│  DiffSummary       │  (optional, non-fatal)
│  (optional)     │    │  short + actions   │
└────────┬────────┘    └────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Storage  ($FLIGHTREC_HOME)         │
│  objects/  snapshots/  diffs/       │
└─────────────────────────────────────┘
         │
         ▼
  flightrec replay / tui / report
```

## Storage layout

```
$FLIGHTREC_HOME/           # default: ~/.flightrec; override: FLIGHTREC_HOME env
├── config.toml
├── objects/
│   └── <2-char prefix>/
│       └── <62-char rest>.blob    # raw file content, SHA-256 addressed
├── snapshots/
│   └── <id>.json                  # SnapshotManifest
└── diffs/
    └── diff-<id>.json             # DiffEvent
```

## Snapshot engine

`src/snapshot.rs` — `take_snapshot(roots, include, exclude, blob_store)`

1. Walks each configured root with `walkdir`, skipping symlinks.
2. Applies include/exclude glob sets (`globset` crate). An empty include list passes all files.
3. For each matched file ≤ 10 MiB: reads full content, SHA-256 hashes it, writes blob.
4. Files > 10 MiB: SHA-256 hashed by streaming read; content **not** stored in blob store.
5. Detects text files by scanning the first 8 KiB for null bytes.
6. Produces a `SnapshotManifest` with a millisecond-precision ID (`YYYYMMDDTHHmmss-mmm`).

### Snapshot manifest (real specimen)

```json
{
  "snapshot_id": "20260612T091500-123",
  "created_at": "2026-06-12T09:15:00.123456+00:00",
  "roots": [
    "~/projects/my-app"
  ],
  "files": [
    {
      "path": "~/projects/my-app/README.md",
      "size": 1024,
      "blob_hash": "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447",
      "is_text": true
    },
    {
      "path": "~/projects/my-app/src/main.rs",
      "size": 512,
      "blob_hash": "3e07c5c2b4e2b3f5a8d1e9f0c6a7b4d2e8f1a3c5d7e9f0b2c4a6d8e0f2b4c6",
      "is_text": true
    }
  ]
}
```

## Blob store

`src/blobstore.rs` — content-addressable, git-style fan-out

- **Layout**: `objects/<first-2-hex-chars>/<remaining-62-hex-chars>.blob`
- **Writes**: atomic (temp file + `fs::rename`); duplicate writes are no-ops.
- **Cap**: only files ≤ 10 MiB are stored. Larger files are hashed but content discarded.
- **No GC**: blobs accumulate until manually cleared. Garbage collection is planned (🚧 see ROADMAP).
- **API**: `has(hash)`, `write(hash, content)`, `read(hash)`, `read_string(hash)`.

Example blob path for hash `a948904f...`:
```
objects/a9/48904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447.blob
```

## Diff engine

`src/diff.rs` — `compute_diff(old, new)` + `enrich_with_diffs(event, store)`

### Change detection

Compares two `SnapshotManifest` values by path:

| Condition | Change type |
|-----------|-------------|
| Path in new only | `added` |
| Path in old only | `removed` |
| Path in both, different `blob_hash` | `modified` |
| Path disappears in old, identical blob appears at new path | `renamed` |

**Rename detection limitation**: uses blob hash equality. If two distinct files happen to have identical content, one may be misidentified as a rename. This is a best-effort heuristic, not a guaranteed rename tracker.

### Unified diffs

`enrich_with_diffs` loads both blobs from the store and calls `unified_diff(old, new)` (via `similar::TextDiff`) to produce `@@`-headed hunks with 3 context lines.

Falls back to a size-only string (`"size: X -> Y bytes"`) when either blob is missing — for example, when replaying diffs generated before v0.2 or for files > 10 MiB.

### DiffEvent (real specimen)

```json
{
  "diff_id": "diff-20260612T091530-456",
  "from_snapshot_id": "20260612T091500-123",
  "to_snapshot_id": "20260612T091530-456",
  "created_at": "2026-06-12T09:15:30.456789+00:00",
  "changes": [
    {
      "path": "~/projects/my-app/README.md",
      "change_type": "modified",
      "old_hash": "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447",
      "new_hash": "f91b49d72a618245617a3a4b1eff3268c91d5a6a21489507ce9b52387a82011c",
      "old_size": 1024,
      "new_size": 1087,
      "diff_text": "--- original\n+++ modified\n@@ -1,4 +1,5 @@\n # my-app\n \n-Short description.\n+Short description.\n+\n+Added installation section.\n",
      "renamed_from": null
    },
    {
      "path": "~/projects/my-app/NOTES.md",
      "change_type": "added",
      "old_hash": null,
      "new_hash": "3e07c5c2b4e2b3f5a8d1e9f0c6a7b4d2e8f1a3c5d7e9f0b2c4a6d8e0f2b4c6",
      "old_size": null,
      "new_size": 64,
      "diff_text": null,
      "renamed_from": null
    }
  ]
}
```

When LLM reporting is enabled, a `summary` field is added:

```json
{
  "summary": {
    "llm_provider": "anthropic",
    "model": "claude-haiku-4-5",
    "generated_at": "2026-06-12T09:15:31.123456+00:00",
    "short": "Updated README with installation section and added a new NOTES file.",
    "actions": [
      "Expanded README.md with installation instructions",
      "Created NOTES.md with initial notes"
    ],
    "intent_guess": "Document project setup"
  }
}
```

## LLM reporter

`src/llm/` — optional narrative layer, non-fatal on failure

### Providers

| Provider key | Endpoint | Auth |
|---|---|---|
| `anthropic` | `https://api.anthropic.com/v1/messages` | `ANTHROPIC_API_KEY` (or `api_key_env`) |
| `openai` | `https://api.openai.com/v1/chat/completions` | `OPENAI_API_KEY` (or `api_key_env`) |
| `openai-compatible` | `{base_url}/chat/completions` | `{api_key_env}` env var |
| `ollama` | `{base_url}/api/chat` (default `http://localhost:11434`) | none |

All providers implement the `LlmProvider` trait (`complete(model, system, user) -> Result<String, LlmError>`). Transport only — prompt construction lives in `llm/prompt.rs`.

### Prompt contract

Changes are sorted by path for determinism, `diff_text` is included when present. The prompt requests a strict-JSON response: `{"short": "...", "actions": [...], "intent_guess": "..."}`. The parser tolerates ` ```json ` fences.

### Failure handling

LLM errors are non-fatal in the watch loop. A warning is logged and the diff event is persisted without a `summary` field. The binary never fails a snapshot cycle because an LLM call failed.

## TUI

`src/tui/` — three-screen navigator (`flightrec tui`, also the default no-arg command)

### Screen map

```
Timeline                    DiffDetail                  FileDiff
──────────────────────────  ──────────────────────────  ──────────────────────────
diff-20260612T091530-456    + NOTES.md                  --- original
  2 changes · 09:15:30      ~ README.md                 +++ modified
                                                        @@ -1,4 +1,5 @@
diff-20260612T090000-001     ← Enter to open             # my-app
  1 change · 09:00:00        ← Esc to go back
                                                         -Short description.
 ← j/k navigate                                         +Short description.
 ← Enter to open            ← j/k navigate              +
 ← r to refresh             ← Enter to open             +Added installation...
 ← q to quit                ← Esc to go back
                                                        ← j/k/↑/↓ scroll
                                                        ← g/G top/bottom
                                                        ← Esc to go back
```

Empty state: "no diffs yet — run `flightrec watch`"

Signal colors: `+` added in green, `-` removed in red, `~` modified in amber. `→` renamed in neutral.

Navigation keys: `j`/`k` or `↑`/`↓` (move), `Enter` (drill in), `Esc` or `Backspace` (back), `g`/`G` (top/bottom), `r` (refresh timeline), `q` (quit).

## CLI commands

| Command | Description |
|---|---|
| `flightrec init` | Write starter `config.toml` in `$FLIGHTREC_HOME`. Flags: `--force`, `--root <path>` |
| `flightrec watch` | Run daemon loop (polling). Flags: `--once`, `--interval <secs>`, `--no-llm` |
| `flightrec diff <a> <b>` | Display diff between two snapshot IDs. Flags: `--json` |
| `flightrec replay` | Print diff history chronologically. Flags: `--since`, `--path` |
| `flightrec report <id>` | Render LLM narrative for a diff. Flags: `--format md\|json` |
| `flightrec tui` | Open interactive TUI (also the default no-arg invocation) |

## Design decisions

- **No GC**: the blob store accumulates indefinitely. This was an explicit v0.2 scope decision — correctness before compaction.
- **Polling only**: the daemon polls on a configurable interval. Event-driven watch via inotify/FSEvents is planned (🚧).
- **Blocking HTTP**: LLM providers use `reqwest` in blocking mode. The watch loop is not async; LLM calls happen inline and add latency to each tick.
- **No webhooks**: webhook output is planned (🚧) but not implemented.
