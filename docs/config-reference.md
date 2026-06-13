# flightrec — Config Reference

Configuration lives at `$FLIGHTREC_HOME/config.toml` (default `~/.flightrec/config.toml`).

Generate a starter file with `flightrec init` — it writes the current defaults and prints the next steps.

---

## `[watch]`

### `watch.roots`

**Type**: `Vec<String>` (array of paths)
**Default**: `["."]` (current working directory at load time)

Directories to watch. Each path is expanded (`~` resolved) and canonicalized at startup. Non-existent roots are silently skipped.

```toml
[watch]
roots = [
  "~/projects/my-app",
  "~/projects/my-agent-config",
]
```

---

## `[filter]`

### `filter.include`

**Type**: `Vec<String>` (array of glob patterns)
**Default**: common source file extensions (`.md`, `.txt`, `.sh`, `.rs`, `.toml`, `.json`, `.yml`, `.yaml`, `.rb`, `.py`, `.ts`, `.tsx`, `.js`)

Glob patterns a file path must match to be included. Applied relative to each watch root. An **empty** list passes all files (no include filtering).

```toml
[filter]
include = [
  "**/*.md",
  "**/*.rs",
  "**/*.toml",
]
```

### `filter.exclude`

**Type**: `Vec<String>` (array of glob patterns)
**Default**: `["**/.git/**", "**/node_modules/**", "**/*.log", "**/.DS_Store", "**/tmp/**", "**/.cache/**", "**/.next/**", "**/target/**"]`

Glob patterns to exclude. Evaluated before `include`. A file matching any exclude pattern is skipped regardless of include rules.

```toml
[filter]
exclude = [
  "**/.git/**",
  "**/node_modules/**",
  "**/target/**",
  "**/*.log",
]
```

---

## `[daemon]`

### `daemon.interval_seconds`

**Type**: `u64` (unsigned integer)
**Default**: `60`

Polling interval in seconds. The watch loop sleeps this many seconds between snapshot cycles. Use `--once` to take a single snapshot and exit.

```toml
[daemon]
interval_seconds = 30
```

---

## `[llm]`

### `llm.enabled`

**Type**: `bool`
**Default**: `false`

Whether to call an LLM provider after each diff cycle to generate a narrative summary. When `false`, `flightrec watch` records diffs without summaries.

```toml
[llm]
enabled = true
```

### `llm.provider`

**Type**: `String`
**Default**: `"anthropic"`

Which LLM backend to use. Supported values:

| Value | Backend |
|---|---|
| `"anthropic"` | Anthropic Messages API |
| `"openai"` | OpenAI Chat Completions API |
| `"openai-compatible"` | OpenAI-compatible endpoint (use with `base_url`) |
| `"ollama"` | Local Ollama server |

```toml
[llm]
provider = "anthropic"
```

### `llm.model`

**Type**: `String`
**Default**: `"claude-haiku-4-5"`

Model name passed to the provider. Must be a model supported by the configured provider.

```toml
[llm]
model = "claude-haiku-4-5"      # Anthropic
# model = "gpt-4o-mini"        # OpenAI
# model = "llama3.2"           # Ollama
```

### `llm.base_url`

**Type**: `Option<String>` (optional)
**Default**: `null` (provider defaults apply)

Override the API endpoint base URL. Required for `"openai-compatible"` providers and local Ollama instances on non-default ports.

- Anthropic default: `https://api.anthropic.com`
- OpenAI default: `https://api.openai.com/v1`
- Ollama default: `http://localhost:11434`

```toml
[llm]
provider = "openai-compatible"
base_url = "http://localhost:8080/v1"
```

### `llm.api_key_env`

**Type**: `Option<String>` (optional)
**Default**: `null` (provider defaults apply: `ANTHROPIC_API_KEY` for anthropic, `OPENAI_API_KEY` for openai)

Name of the environment variable that holds the API key. Override this when using a custom provider or a key stored under a non-default variable name.

```toml
[llm]
api_key_env = "MY_LLM_API_KEY"
```

### `llm.max_changes_per_prompt`

**Type**: `usize` (unsigned integer)
**Default**: `30`

Maximum number of file changes included in a single LLM prompt. Changes beyond this limit are omitted from that call. Reduce this value if you hit token limits with large diff cycles.

```toml
[llm]
max_changes_per_prompt = 20
```

---

## `[output]`

### `output.json_log_dir`

**Type**: `String` (path)
**Default**: `"~/.flightrec/logs"`

**Reserved — not yet wired.** This field is parsed and stored but is not currently used by the storage layer. Setting it has no effect in v0.2. It is reserved for a future structured JSON log output feature.

```toml
[output]
json_log_dir = "~/.flightrec/logs"
```

---

## Full example

```toml
[watch]
roots = [
  "~/projects/my-app",
]

[filter]
include = [
  "**/*.md",
  "**/*.rs",
  "**/*.toml",
  "**/*.json",
]
exclude = [
  "**/.git/**",
  "**/target/**",
  "**/node_modules/**",
]

[daemon]
interval_seconds = 60

[llm]
enabled = true
provider = "anthropic"
model = "claude-haiku-4-5"
max_changes_per_prompt = 30

[output]
json_log_dir = "~/.flightrec/logs"
```
