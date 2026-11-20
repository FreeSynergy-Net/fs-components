//! The style sheet of the tree.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the tree: only theme variables. A picked item is told by ground and weight; the cursor has an edge; the mark of a parent is
/// a triangle (right closed, down open), so open and closed are not told by colour.
pub const STYLES: &str = r"
.fs-tree-root { display: flex; flex-direction: column; padding: var(--space-1); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-tree-root:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-treeitem-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-treeitem-row:hover { background: var(--accent-subtle); }
.fs-treeitem-mark { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-treeitem-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-treeitem-picked { background: var(--accent-subtle); }
.fs-treeitem-picked .fs-treeitem-label { font-weight: 600; }
.fs-treeitem-current { border-color: var(--border-focus); }
.fs-treeitem { display: flex; flex-direction: column; }
.fs-treeitem-group { display: flex; flex-direction: column; }
";
