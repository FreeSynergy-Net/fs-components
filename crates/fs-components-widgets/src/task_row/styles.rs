//! The style sheet of the task row.
//!
//! Pattern: none - only theme variables (constants, no behaviour).

/// CSS of the task row: only theme variables. Done is told by the tick and the strike-through, the priority by its word and its colour.
pub const STYLES: &str = r"
.fs-task-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-task-row:hover { background: var(--accent-subtle); }
.fs-task-row:focus { border-color: var(--border-focus); }
.fs-task-row-selected { background: var(--accent-subtle); }
.fs-task-row-muted .fs-task-row-title { color: var(--text-secondary); }
.fs-task-row-body { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); flex-grow: 1; min-width: 0; }
.fs-task-row-main { display: flex; flex-direction: column; min-width: 0; flex-grow: 1; }
.fs-task-row-title { color: var(--text-primary); font-size: var(--text-sm); }
.fs-task-row-done .fs-task-row-title { color: var(--text-secondary); text-decoration: line-through; }
.fs-task-row-sub { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-task-row-facts { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); }
.fs-task-row-prio { padding: 0 var(--space-1); border-radius: var(--radius-default); background: var(--accent-subtle); color: var(--text-primary); font-size: var(--text-xs); }
.fs-task-row-prio-warning { background: var(--warning-bg); }
.fs-task-row-prio-danger { background: var(--error-bg); }
.fs-task-row-due { color: var(--text-secondary); font-size: var(--text-xs); font-style: italic; }
.fs-task-row-more { color: var(--text-secondary); font-size: var(--text-sm); }
";
