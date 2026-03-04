# agentscope — Technical Specification

## 1) Problem statement
AI agents now operate directly inside VMs and containers, editing code, configs, and operational artifacts at high speed. The current tooling stack (git history, CLI logs, task runners) only captures fragments of that activity. As a result, teams lack a deterministic, structured record of *what changed, when it changed, and how it changed* — especially for non‑git files and transient automation.

`agentscope` solves this by continuously snapshotting the filesystem, diffing snapshots into structured change events, and producing a concise narrative that explains what the agent did. This enables observability, accountability, and replay of agentic behavior across codebases and runtime environments without invasive instrumentation in each tool.

## 2) Architecture

### 2.1 Daemon design
- **Long‑running Rust daemon** with configurable polling interval (default: 60s).
- **Snapshot engine** that walks configured directories, hashes file contents, and stores content‑addressable blobs + snapshot manifests.
- **Diff engine** computes changes between consecutive snapshots.
- **LLM reporter** converts diff batches to narrative summaries.
- **Output layer** writes structured JSON logs + optional webhooks.

Runtime loop:
1. Load config
2. Collect snapshot
3. Compute diff from previous snapshot
4. Persist diff event + narrative (if enabled)
5. Emit webhook (optional)
6. Sleep until next tick

### 2.2 Snapshot format (git‑like)
- **Blob store**: each file content hashed (SHA‑256) into a blob store: `<hash>.blob`
- **Tree manifest**: maps path → blob hash + metadata
- **Snapshot manifest**: top‑level metadata referencing tree and timestamp
- **Storage**: local directory under `~/.agentscope/`

### 2.3 Diff algorithm
- Compare snapshot manifests keyed by path.
- For each path:
  - **Added**: path exists in new snapshot only
  - **Removed**: path exists in old snapshot only
  - **Modified**: path exists in both, but blob hash differs
  - **Renamed** (best effort): identical blob hash appears at new path and disappears at old path in same diff cycle
- For modified files, compute **line‑level diff** (unified diff) for text; record binary delta metadata for non‑text.

### 2.4 LLM integration layer
- Batches diff events into prompt‑sized chunks (target 10–50 changes per call).
- Uses a deterministic prompt template to summarize actions into:
  - **Short summary** (1–3 sentences)
  - **Action list** (bullet points)
  - **Intent guess** (if feasible)
- Supports multiple providers via adapters (OpenAI, Anthropic, local models).
- LLM output stored alongside diff events for auditability.

## 3) Data format

### 3.1 Snapshot manifest (JSON)
```json
{
  "snapshot_id": "2026-03-04T19:26:00Z-000123",
  "created_at": "2026-03-04T19:26:00Z",
  "root": "/home/zakelfassi",
  "tree_hash": "sha256:1d90...",
  "files": [
    {
      "path": "/home/zakelfassi/clawd/README.md",
      "mode": "100644",
      "size": 2048,
      "mtime": "2026-03-04T19:25:12Z",
      "blob_hash": "sha256:aa3b...",
      "is_text": true,
      "line_count": 74
    }
  ]
}
```

### 3.2 Diff event (JSON)
```json
{
  "diff_id": "diff-000124",
  "from_snapshot": "2026-03-04T19:25:00Z-000122",
  "to_snapshot": "2026-03-04T19:26:00Z-000123",
  "created_at": "2026-03-04T19:26:02Z",
  "changes": [
    {
      "path": "/home/zakelfassi/clawd/README.md",
      "change_type": "modified",
      "old": { "blob_hash": "sha256:111...", "size": 1980 },
      "new": { "blob_hash": "sha256:aa3b...", "size": 2048 },
      "diff": "@@ -1,3 +1,5 @@\n ...",
      "is_text": true,
      "renamed_from": null
    },
    {
      "path": "/home/zakelfassi/clawd/newfile.txt",
      "change_type": "added",
      "new": { "blob_hash": "sha256:bb8c...", "size": 122 },
      "is_text": true
    }
  ],
  "summary": {
    "llm_provider": "openai",
    "model": "gpt-5.2",
    "generated_at": "2026-03-04T19:26:05Z",
    "short": "Updated project README and added a new note file.",
    "actions": [
      "Expanded README with installation steps",
      "Added newfile.txt with draft notes"
    ],
    "intent_guess": "Document project usage"
  }
}
```

## 4) Filtering strategy

### 4.1 Default rules
**Include roots:**
- `~/.openclaw/`
- `/home/zakelfassi/clawd/`
- `/home/zakelfassi/tac-monorepo/`

**Exclude patterns (default):**
- `**/.git/**`
- `**/node_modules/**`
- `**/*.log`
- `**/.DS_Store`
- `**/.tmp/**`
- `**/tmp/**`
- `**/.cache/**`

### 4.2 Config format
TOML in `~/.agentscope/config.toml`:
```toml
[watch]
roots = [
  "/home/zakelfassi/.openclaw",
  "/home/zakelfassi/clawd",
  "/home/zakelfassi/tac-monorepo"
]

[filter]
include = ["**/*.md", "**/*.rs", "**/*.toml", "**/*.json", "**/*.yml"]
exclude = ["**/.git/**", "**/node_modules/**", "**/*.log", "**/.cache/**"]

[daemon]
interval_seconds = 60

[llm]
enabled = true
provider = "openai"
model = "gpt-5.2"
max_changes_per_prompt = 30

[output]
json_log_dir = "~/.agentscope/logs"
webhook_url = ""
```

## 5) Trigger modes

1. **Polling interval (default):** daemon ticks every N seconds, snapshots and diffs.
2. **On‑demand CLI trigger:** `agentscope watch --once` or `agentscope snapshot`.
3. **Webhook/event trigger (Phase 2):** external systems can POST to `/trigger` to force a snapshot, e.g. CI finished, agent run complete.

## 6) LLM integration

### 6.1 Diff batching
- Group changes by directory and by type (code vs config vs notes).
- Max changes per prompt configurable.
- Skip sending binary diffs unless change summary required (binary metadata only).

### 6.2 Prompt design (template)
```
SYSTEM: You are an observability analyst. Summarize filesystem changes clearly and tersely.
USER:
Changes from snapshot A → B:
- path: <path>
  change: <added|removed|modified|renamed>
  diff: <unified diff or summary>

Output JSON with keys: short, actions, intent_guess.
```

### 6.3 Output format
LLM output stored as part of diff event and optionally as standalone report:
```json
{ "short": "...", "actions": ["..."], "intent_guess": "..." }
```

## 7) CLI interface

- `agentscope watch`
  - Runs daemon loop; respects config.
  - Flags: `--interval`, `--once`, `--no-llm`.

- `agentscope diff <snapshot_id_a> <snapshot_id_b>`
  - Prints diff JSON or human view.
  - Flags: `--json`, `--summary`.

- `agentscope replay`
  - Timeline view of diffs, optionally filtered by path.
  - Flags: `--since`, `--until`, `--path`.

- `agentscope report <diff_id>`
  - Renders LLM narrative for a diff or range.
  - Flags: `--format md|json`.

## 8) Phase 1 implementation plan (Rust)

1. **Bootstrap crate**
   - `cargo new agentscope` + workspace layout
   - Add deps: `serde`, `serde_json`, `walkdir`, `globset`, `sha2`, `time`, `tokio`, `clap`
2. **Config loader**
   - TOML parsing via `toml` crate
   - Expand `~` and validate paths
3. **Snapshot engine**
   - Walk directories
   - Apply include/exclude filters
   - Hash files, write blobs
   - Produce snapshot manifest
4. **Diff engine**
   - Compare two manifests
   - Compute added/removed/modified/renamed
   - Generate unified diffs for text
5. **Storage layer**
   - `~/.agentscope/objects/` for blobs
   - `~/.agentscope/snapshots/` for manifests
   - `~/.agentscope/diffs/` for diff events
6. **Daemon loop**
   - Polling tick
   - Persist diffs
   - Handle `--once`
7. **LLM adapter (optional in Phase 1)**
   - Simple HTTP client + provider config
   - Prompt + JSON parsing
8. **CLI interface**
   - `watch`, `diff`, `replay`, `report` commands
9. **Tests**
   - Unit tests for diff detection
   - Golden snapshot fixture tests
10. **Docs**
   - README + config reference

## 9) Phase 2+ roadmap

- **Phase 2: Generic daemon**
  - Multiple watch profiles
  - Event‑driven triggers via FS notifications (inotify/FSEvents)
  - Webhook server for external triggers

- **Phase 3: Web UI**
  - Timeline view of diffs
  - Drill‑down on file changes
  - Query by path, agent, run ID

- **Phase 4: Paid product**
  - Hosted diff storage
  - Team dashboards + RBAC
  - Compliance exports

- **Phase 5: Robotics / edge agents**
  - Lightweight snapshot delta streaming
  - On‑device buffer + periodic sync
  - Offline mode with replay

## 10) Repo structure

```
agentscope/
  README.md
  SPEC.md
  Cargo.toml
  src/
    main.rs
    cli/
      mod.rs
      watch.rs
      diff.rs
      replay.rs
      report.rs
    config/
      mod.rs
      schema.rs
    snapshot/
      mod.rs
      manifest.rs
      hasher.rs
    diff/
      mod.rs
      compute.rs
      unified.rs
    llm/
      mod.rs
      providers/
        openai.rs
        anthropic.rs
    storage/
      mod.rs
      blobs.rs
      snapshots.rs
      diffs.rs
    utils/
      path.rs
      time.rs
  tests/
    fixtures/
    diff_basic.rs
```
