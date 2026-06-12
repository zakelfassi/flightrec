# ollama-local

Generate diff narratives with a local Ollama model — no API key, no data
leaving your machine.

**Why**: Air-gapped environments or strict data-residency requirements can
still get LLM summaries without sending data to a cloud provider.

## Prerequisites

```sh
brew install ollama && ollama serve &
ollama pull llama3.2
```

## Run it

```sh
cp config.toml ~/.flightrec/config.toml
# Edit watch.roots
flightrec watch --once
```

Expected output:

```
[2026-06-12T09:15:30.456+00:00] snapshot 20260612T091530-456 — 12 files → ~/.flightrec/snapshots/20260612T091530-456.json
  1 changes:
    ~ ~/projects/my-app/src/main.rs
  [llm] summary generated (ollama/llama3.2)
```

## Change the model

Set `llm.model` to any model pulled into Ollama (`qwen2.5-coder`, `mistral`, etc.).
`base_url` defaults to `http://localhost:11434`.
