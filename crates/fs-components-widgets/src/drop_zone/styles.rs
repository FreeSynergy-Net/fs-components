//! The style sheet of the drop zone.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the drop zone: only theme variables. Over it the edge and the ground change, and the words change too (not colour alone).
pub const STYLES: &str = r"
.fs-drop-zone { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-2); padding: var(--space-6); border-radius: var(--radius-default); border-width: 2px; border-color: var(--border); background: var(--surface); cursor: pointer; }
.fs-drop-zone:hover { background: var(--accent-subtle); }
.fs-drop-zone:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-drop-zone-over { border-color: var(--accent); background: var(--accent-subtle); }
.fs-drop-zone-text { color: var(--text-primary); font-size: var(--text-sm); }
";
