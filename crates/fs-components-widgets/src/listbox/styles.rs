//! The style sheet of the listbox.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the listbox: only theme variables. A picked option is told by ground and weight (not by colour alone); the cursor has an
/// edge; a disabled option is weak and the pointer ignores it.
pub const STYLES: &str = r"
.fs-listbox { display: flex; flex-direction: column; padding: var(--space-1); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-listbox:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-option { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-option:hover { background: var(--accent-subtle); }
.fs-option-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-option-picked { background: var(--accent-subtle); }
.fs-option-picked .fs-option-label { font-weight: 600; }
.fs-option-current { border-color: var(--border-focus); }
.fs-option-disabled { cursor: default; }
.fs-option-disabled .fs-option-label { color: var(--text-tertiary); }
";
