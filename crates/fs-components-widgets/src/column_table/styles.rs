//! The style sheet of the column table: only `.fs-ct-*` rules and theme variables.
//!
//! Pattern: none.

/// The sheet.
pub const STYLES: &str = r"
.fs-ct { gap: var(--space-2); padding: var(--space-3); }
.fs-ct-head { align-items: center; gap: var(--space-3); padding: 0 var(--space-2); }
.fs-ct-head-cell { flex-basis: 140px; flex-grow: 1; color: var(--text-secondary); font-size: 14px; font-weight: 600; }
.fs-ct-row { gap: var(--space-3); padding: var(--space-2); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface-elevated); }
.fs-ct-handle { width: 24px; flex-shrink: 0; color: var(--text-secondary); font-size: 18px; }
.fs-ct-cell { flex-basis: 140px; flex-grow: 1; min-width: 0; font-size: 15px; }
.fs-ct-status { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--accent-subtle); }
.fs-ct-rowbtn { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); color: var(--text-secondary); font-size: 13px; }
.fs-ct-rowbtn:hover { background: var(--accent-subtle); }
.fs-ct-problem { color: var(--error); font-size: 13px; }
.fs-ct-add { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); color: var(--text-secondary); font-size: 15px; }
.fs-ct-add:hover { background: var(--accent-subtle); color: var(--text-primary); }
.fs-ct-foot { gap: var(--space-3); padding-top: var(--space-3); }
.fs-ct-reason { color: var(--text-secondary); font-size: 14px; }
";
