//! The style sheet of the table.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the table: only theme variables. The head is told by weight and a line under it (not by colour alone).
pub const STYLES: &str = r"
.fs-table { display: flex; flex-direction: column; border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-table-caption { padding: var(--space-2) var(--space-3); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-table-head { display: flex; flex-direction: row; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-table-row { display: flex; flex-direction: row; }
.fs-table-row:hover { background: var(--accent-subtle); }
.fs-table-cell { padding: var(--space-1) var(--space-3); }
.fs-table-cell-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-table-headcell-text { color: var(--text-primary); font-size: var(--text-sm); font-weight: 700; }
.fs-table-rowhead-text { color: var(--text-primary); font-size: var(--text-sm); font-weight: 600; }
";
