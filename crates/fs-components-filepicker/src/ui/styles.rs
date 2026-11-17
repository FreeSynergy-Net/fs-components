//! The style sheet of the file chooser parts.
//!
//! Pattern: none — only theme variables (constants, no behaviour). The window adds it to its sheets next to the others.

/// CSS of the pick button and the path field view: only theme variables.
pub const STYLES: &str = r"
.fs-fp { display: flex; flex-direction: column; align-self: flex-start; gap: var(--space-2); }
.fs-fp-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); }
.fs-fp-state { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-fp-problem { color: var(--danger-text); font-size: var(--text-sm); }
.fs-fp-provisional { color: var(--text-secondary); font-size: var(--text-xs); font-style: italic; }
.fs-fp-prompt { display: flex; flex-direction: column; width: 100%; max-width: 420px; gap: var(--space-2); padding: var(--space-3); border-radius: var(--radius-lg); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-fp-title { color: var(--text-primary); font-size: var(--text-lg); }
.fs-fp-buttons { display: flex; flex-direction: row; justify-content: flex-end; gap: var(--space-2); }
";
