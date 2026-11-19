//! The style sheet of the help.
//!
//! Pattern: none — only theme variables (constants, no behaviour). The pairs (text on the surface) are measured in every style by
//! `tests/help_contrast.rs`, read from this sheet.

/// CSS of the help: only theme variables. A missing or unavailable help has the edge of a warning (and says so in words).
pub const STYLES: &str = r"
.fs-help { display: flex; flex-direction: column; gap: var(--space-2); padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-help-found { border-left-width: 1px; }
.fs-help-missing, .fs-help-unavailable { border-left-width: var(--toast-edge); border-left-color: var(--warning); }
.fs-help-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-help-note { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-help-action { display: flex; flex-direction: row; }
";
