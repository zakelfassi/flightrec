# Roadmap

## Shipped — v0.2

- **Polling daemon** — configurable interval watch loop (`flightrec watch`, `--once` for single-shot runs)
- **Content-addressable blob store** — git-style fan-out under `$FLIGHTREC_HOME/objects/`, atomic writes, deduplication, 10 MiB cap
- **Snapshot engine** — walks configured roots, applies include/exclude globs, produces JSON manifests
- **Unified line diffs** — real `@@`-headed hunks via the `similar` crate; falls back to size-only for pre-blob or binary files
- **Rename detection** — best-effort: same blob hash at a new path in the same diff cycle
- **LLM narratives** — Anthropic, OpenAI, OpenAI-compatible, and Ollama providers; non-fatal in the watch loop
- **TUI** — three-screen ratatui navigator: Timeline → DiffDetail → FileDiff; vim keys
- **`flightrec init`** — writes a starter config with the current directory as watch root
- **`flightrec replay`** — chronological diff history in the terminal
- **`flightrec report`** — renders the LLM narrative for a specific diff
- **`FLIGHTREC_HOME` env override** — governs both storage and config location; enables hermetic tests

## Next

- **Event-driven watch** (inotify on Linux, FSEvents on macOS) — eliminates the polling gap so changes are recorded in real time rather than on the next tick
- **Webhooks** — POST a structured payload to a configured URL after each diff cycle; enables CI/CD and agent-orchestration integrations
- **Blob GC** — compact the object store by removing blobs no longer referenced by any snapshot; `flightrec gc` command

## Later

- **Web timeline UI** — browser-based diff explorer with path filtering, date range, and side-by-side file diffs; served by a local HTTP server embedded in the binary

## Exploratory

- **Hosted storage** — optional cloud sync of snapshots and diffs; useful for teams auditing shared agent workloads without managing a central server themselves

## Further

- **Edge and robotics** — lightweight delta streaming for constrained devices; on-device buffer with periodic sync and offline replay; same wire format, smaller footprint
