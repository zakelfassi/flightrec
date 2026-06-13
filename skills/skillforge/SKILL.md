---
name: skillforge
description: Scaffold a new reusable skill (SKILL.md + optional scripts/references). Use when asked to create a skill, when a task would benefit from a reusable procedural guide, or when you notice repeated workflow friction.
metadata:
  spec: agentskills.io
---

# SkillForge

Create well-formed, agentskills.io-compliant skills quickly and consistently.

## Inputs

- Description of the workflow pattern to encode
- Intended consumer (agent harness, human, CI)

## Outputs

- `skills/<skill-name>/SKILL.md`
- Optionally: `skills/<skill-name>/run.sh`, `references/`, `assets/`
- Updated `.skills-registry.md` entry

## Steps

1. **Name the pattern** — one responsibility, `kebab-case`, verb-led when possible.
2. **Create the directory** — `skills/<skill-name>/` at the project root.
3. **Write `SKILL.md`** — frontmatter: `name` + `description` only (plus optional `metadata` block). Body: numbered steps, crisp examples. Keep under 200 lines.
4. **Add `run.sh`** if the skill involves runnable automation. Mark executable (`chmod +x`).
5. **Register** — add a row to `.skills-registry.md` at the project root.

## Minimal SKILL.md skeleton

```markdown
---
name: your-skill-name
description: What it does. Use when <trigger>.
---

# Your Skill Name

## Steps
1. ...
```

## Guardrails

- Frontmatter keys: `name`, `description`, and optionally `metadata`. Drop nonstandard keys like `triggers:`.
- No hardcoded paths, secrets, or personal data.
- Do not over-forge: only create a skill when it will be reused or meaningfully reduces future context.
