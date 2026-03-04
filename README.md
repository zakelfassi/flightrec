# agentscope

**Git‑like observability for AI agents.**

`agentscope` is a Rust daemon that snapshots the filesystem, diffs changes over time, and produces a deterministic, LLM‑readable record of what an agent did — plus a human‑readable narrative. Think *New Relic for agentic work* across code, configs, and local automation.

## Why it exists
AI agents act fast and edit lots of files, but there’s no trustworthy, structured record of *what changed, when, and why*. `agentscope` provides:

- **Content‑addressable snapshots** (git‑style)
- **Structured diffs** between snapshots
- **LLM narratives** that summarize intent and actions
- **Webhooks & JSON logs** for downstream systems

## Phase 1 scope (OpenClaw native)
- Watches: `~/.openclaw/`, `/home/zakelfassi/clawd/`, `/home/zakelfassi/tac-monorepo/`
- Filters noise (`.git/`, `node_modules/`, `*.log`, temp files)
- CLI triggers + polling loop

## CLI (planned)
- `agentscope watch` — run daemon and collect snapshots
- `agentscope diff` — compare two snapshots
- `agentscope replay` — timeline view of changes
- `agentscope report` — generate LLM narrative

## Status
Spec complete. Implementation in progress.
