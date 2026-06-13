//! Design-system signal colors for the flightrec TUI.
//!
//! These constants are the canonical truecolor values shared across all
//! flightrec surfaces: TUI (here), landing page (`landing/`), and brand
//! assets (`assets/BRAND.md`). They encode change-state semantics — never
//! used decoratively.
//!
//! Cross-surface palette (grep anchor: `#1A9E55`, `#D43B3B`, `#C7860A`):
//!
//! | Token    | Hex       | Semantic              |
//! |----------|-----------|-----------------------|
//! | ADDED    | `#1A9E55` | added file or line    |
//! | REMOVED  | `#D43B3B` | removed file or line  |
//! | MODIFIED | `#C7860A` | modified file         |
//! | RENAMED  | —         | renamed (ink-soft)    |

use ratatui::style::Color;

/// Signal green `#1A9E55` — added files and `+` diff lines.
///
/// Matches the design-system `added` token used in landing/ and assets/.
pub const ADDED: Color = Color::Rgb(0x1A, 0x9E, 0x55);

/// Signal red `#D43B3B` — removed files and `−` diff lines.
///
/// Matches the design-system `removed` token used in landing/ and assets/.
pub const REMOVED: Color = Color::Rgb(0xD4, 0x3B, 0x3B);

/// Signal amber `#C7860A` — modified files and `@@` hunk headers.
///
/// Matches the design-system `modified` token used in landing/ and assets/.
pub const MODIFIED: Color = Color::Rgb(0xC7, 0x86, 0x0A);

/// Neutral — renamed files (ink-soft; no dedicated signal hex).
///
/// Renamed entries use a muted neutral rather than a signal color because
/// renaming is a structural change, not an additive/destructive one.
pub const RENAMED: Color = Color::Rgb(0x52, 0x52, 0x4E);
