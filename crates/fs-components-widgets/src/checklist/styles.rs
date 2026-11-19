//! The style sheet of the checklist.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the checklist: only theme variables. Done is told by the box (a tick) and the weight, not by colour alone; the cursor has an edge.
pub const STYLES: &str = r"
.fs-checklist { display: flex; flex-direction: column; gap: var(--space-1); }
.fs-checklist:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-check-item { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-check-item:hover { background: var(--accent-subtle); }
.fs-check-item-current { border-color: var(--border-focus); }
.fs-check-box { color: var(--text-primary); font-size: var(--text-base); }
.fs-check-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-check-item-done .fs-check-text { color: var(--text-secondary); text-decoration: line-through; }
.fs-check-add-box { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-check-remove { padding: 0px var(--space-1); border-radius: var(--radius-full); cursor: pointer; }
.fs-check-remove:hover { background: var(--accent-subtle); }
.fs-check-remove-mark { color: var(--text-secondary); font-size: var(--text-sm); }
";
