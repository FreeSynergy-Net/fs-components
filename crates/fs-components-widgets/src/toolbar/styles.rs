//! The style sheet of the toolbar.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the toolbar: only theme variables. A pressed toggle is told by its ground and its weight (not by colour alone); the
/// cursor has an edge; a disabled control is weak and ignores the pointer.
pub const STYLES: &str = r"
.fs-toolbar { display: flex; flex-direction: row; align-items: center; gap: var(--space-1); padding: var(--space-1); border-radius: var(--radius-default); background: var(--surface); }
.fs-toolbar-vertical { flex-direction: column; align-items: stretch; }
.fs-toolbar:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-tool { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-tool:hover { background: var(--accent-subtle); }
.fs-tool-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tool-on { background: var(--accent-subtle); }
.fs-tool-on .fs-tool-label { font-weight: 600; }
.fs-tool-current { border-color: var(--border-focus); }
.fs-tool-disabled { cursor: default; }
.fs-tool-disabled .fs-tool-label { color: var(--text-tertiary); }
.fs-tool-separator { width: 1px; align-self: stretch; background: var(--border); }
";
