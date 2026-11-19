//! The style sheet of the disclosure and the accordion.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of both: only theme variables. Open is told by the mark (a different triangle) and the weight, not by colour alone; the
/// header that has the keyboard has an edge.
pub const STYLES: &str = r"
.fs-disclosure { display: flex; flex-direction: column; border-radius: var(--radius-default); }
.fs-disc-head { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-disc-head:hover { background: var(--accent-subtle); }
.fs-disc-head:focus { border-color: var(--border-focus); }
.fs-disc-mark { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-disc-title { color: var(--text-primary); font-size: var(--text-base); }
.fs-disclosure-open .fs-disc-title { font-weight: 600; }
.fs-disc-head-off { cursor: default; }
.fs-disc-panel { padding: var(--space-3); color: var(--text-primary); }
.fs-accordion { display: flex; flex-direction: column; gap: var(--space-1); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); }
";
