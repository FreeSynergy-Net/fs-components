//! The style sheet of the person picker.
//!
//! Pattern: none, only theme variables (constants, no behaviour).

/// CSS of the person picker: only theme variables.
pub const STYLES: &str = r"
.fs-pp { display: flex; flex-direction: column; width: 100%; max-width: 520px; gap: var(--space-3); padding: var(--space-4); border-radius: var(--radius-lg); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-pp-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
.fs-pp-title { color: var(--text-primary); font-size: var(--text-lg); }
.fs-pp-list { display: flex; flex-direction: column; gap: var(--space-1); min-height: 0; }
.fs-pp-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); min-width: 0; overflow: hidden; padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: var(--surface); }
.fs-pp-row-cursor { border-color: var(--border-focus); }
.fs-pp-row-on { background: var(--accent-subtle); }
.fs-pp-mark { flex-shrink: 0; width: 16px; height: 16px; border-radius: var(--radius-sm); border-width: 2px; border-color: var(--border); }
.fs-pp-mark-on { background: var(--primary); border-color: var(--primary); }
.fs-pp-pick { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); flex-grow: 1; min-width: 0; overflow: hidden; }
.fs-pp-who { display: flex; flex-direction: column; flex-grow: 1; min-width: 0; overflow: hidden; }
.fs-pp-name { color: var(--text-primary); }
.fs-pp-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-pp-note { color: var(--text-secondary); font-size: var(--text-sm); padding: var(--space-1) var(--space-2); }
.fs-pp-notice { color: var(--error); font-size: var(--text-sm); padding: var(--space-1) var(--space-2); }
.fs-pp-foot { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
.fs-pp-buttons { display: flex; flex-direction: row; gap: var(--space-2); }
";
