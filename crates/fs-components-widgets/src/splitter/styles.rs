//! The style sheet of the splitter.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the splitter: only theme variables. The divider is a bar that gets thicker edges when it has the keyboard or is dragged.
pub const STYLES: &str = r"
.fs-splitter { display: flex; flex-direction: row; }
.fs-splitter-horizontal { flex-direction: column; }
.fs-splitter-pane { overflow: clip; }
.fs-splitter-bar { background: var(--border); border-width: 2px; border-color: var(--border); cursor: col-resize; }
.fs-splitter-bar:hover { background: var(--accent-subtle); }
.fs-splitter-bar:focus { border-color: var(--border-focus); outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-splitter-bar-active { background: var(--accent); }
";
