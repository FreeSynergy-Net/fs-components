//! The style sheet of the chip row.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the chip row: only theme variables. The chip under the cursor has an edge.
pub const STYLES: &str = r"
.fs-chips { display: flex; flex-direction: row; flex-wrap: wrap; gap: var(--space-2); padding: var(--space-1); }
.fs-chips:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-chip { display: flex; flex-direction: row; align-items: center; gap: var(--space-1); padding: var(--space-1) var(--space-2); border-radius: var(--radius-full); border-width: 2px; border-color: transparent; background: var(--accent-subtle); }
.fs-chip-current { border-color: var(--border-focus); }
.fs-chip-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-chip-remove { padding: 0px var(--space-1); border-radius: var(--radius-full); cursor: pointer; }
.fs-chip-remove:hover { background: var(--surface); }
.fs-chip-remove-mark { color: var(--text-secondary); font-size: var(--text-sm); }
";
