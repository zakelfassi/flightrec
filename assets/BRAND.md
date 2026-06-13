# flightrec — Design Tokens

Reference for all visual contributors. If you are changing TUI colors, SVG assets, or any visual output, consult this file first.

---

## Palette — Light

| Token | Hex | Usage |
|-------|-----|-------|
| `paper` | `#FAFAF7` | Page / canvas background |
| `raised` | `#F2F2EE` | Cards, inset panels |
| `ink` | `#111110` | Primary text |
| `ink-soft` | `#52524E` | Secondary text, renamed-change symbol |
| `ink-faint` | `#A3A39C` | Tertiary text, disabled states |
| `rule` | `#DCDCD5` | Hairline borders, dividers |

## Palette — Dark

| Token | Hex | Usage |
|-------|-----|-------|
| `bg` | `#101010` | Page / canvas background |
| `fg` | `#E8E8E2` | Primary text |
| `rule` | `#2A2A28` | Hairline borders, dividers |

---

## Signal Colors (change-state ONLY)

Signal colors are **reserved for change-state semantics**. Do not repurpose them for decoration, emphasis, or branding.

| State | Display hex | Text-safe variant | Symbol |
|-------|-------------|-------------------|--------|
| Added | `#1A9E55` | `#157A43` | `+` |
| Removed | `#D43B3B` | `#B32D2D` | `-` |
| Modified | `#C7860A` | `#9A6A08` | `~` |
| Renamed | ink-soft (`#52524E`) | — | `→` |

---

## Typography

| Role | Family | Weight |
|------|--------|--------|
| Display headings | Space Grotesk | 700 |
| Sub-headings | Space Grotesk | 500 |
| Body text | Space Grotesk | 400 |
| Data, labels, code | IBM Plex Mono | 400 |

Fonts are self-hosted as WOFF2 subsets. No CDN. No third-party font requests.

---

## Layout

- **Grid**: 8px base unit
- **Max width**: 1080px
- **Border radius**: 0 (zero — no rounding)
- **Shadows**: none
- **Borders**: 1px hairline rules only

---

## Motifs

- **Oscilloscope trace**: signature decorative motif (step-function line)
- **Corner registration marks**: technical-drawing aesthetic on figure plates
- **Figure plates**: captions use `FIG. NN — LABEL` in IBM Plex Mono, uppercase

---

## Copy conventions

- No emoji in any text output or documentation
- No exclamation marks in copy
- Landing page makes zero third-party requests (self-hosted assets only)
