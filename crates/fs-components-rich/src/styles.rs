//! The style sheet of the writing surface.
//!
//! Pattern: none — only theme variables (`var(--...)`), no literal colours, sizes or fonts; the style of the process (Standard,
//! Craft) decides how a corner, a frame and a ground look. `--cursor` is the token of the text cursor and `--code-bg`
//! the ground of a code block (the token of every widget; a style may give them; the primary colour and the faint accent are
//! the fallbacks).

/// CSS for the writing surface; added to the theme by the shell (next to the sheets of `fs-components-widgets`).
pub const STYLES: &str = r"
.fs-re { width: 100%; }
.fs-re-code { background: var(--code-bg, var(--accent-subtle)); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); }
.fs-re-bar { background: var(--border); border-radius: var(--radius-sm); }
.fs-re-rule { height: 1px; background: var(--border); }
.fs-re-box { border-width: 1.5px; border-color: var(--text-secondary); background: var(--surface); border-radius: var(--radius-sm); display: flex; flex-direction: row; align-items: center; justify-content: center; }
.fs-re-box-on { border-color: var(--primary); background: var(--primary); }
.fs-re-caret { background: var(--cursor, var(--primary)); }
";
