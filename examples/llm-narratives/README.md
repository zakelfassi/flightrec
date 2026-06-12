# llm-narratives

Add human-readable summaries to every diff using Anthropic claude-haiku.

**Why**: Structured diffs show what changed; LLM narratives explain why it
matters. Each diff event gains a `summary` field — useful for audit logs.

## Prerequisites

Set `ANTHROPIC_API_KEY` in your environment before running.

## Run it

```sh
cp config.toml ~/.flightrec/config.toml
# Edit watch.roots
flightrec watch --once
```

Expected output:

```
[2026-06-12T09:15:30.456+00:00] snapshot 20260612T091530-456 — 12 files → ~/.flightrec/snapshots/20260612T091530-456.json
  2 changes:
    ~ ~/projects/my-app/README.md
    + ~/projects/my-app/CHANGELOG.md
```

The watch command does not print a confirmation line when a summary is generated. On success, the `summary` field is written into the saved diff JSON. On failure, a warning is printed to stderr (`warning: LLM summarization failed …`). Use `flightrec report <diff-id>` to retrieve the narrative.

## View a narrative

```sh
flightrec report diff-20260612T091530-456
# → "Updated README and added CHANGELOG. Actions: expanded README, created CHANGELOG.md"
```

LLM failures are non-fatal — the diff is saved without a summary.
