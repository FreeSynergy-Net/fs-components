//! The style sheet of the grid and the treegrid.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of both: only theme variables. The head is told by weight (not by colour alone), the cursor cell has an edge, an open parent
/// row has its triangle in the first cell; with `cells-hold` the cell has room for an element, the cell the keys went into has the
/// colour of the primary edge, and a spanning head is raised.
pub const STYLES: &str = r"
.fs-grid { display: flex; flex-direction: column; border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-grid:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-grid-row { display: flex; flex-direction: row; }
.fs-grid-row:hover { background: var(--accent-subtle); }
.fs-grid-head { display: flex; flex-direction: row; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-grid-cell { padding: var(--space-1) var(--space-3); border-width: 2px; border-color: transparent; }
.fs-grid-cell-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-grid-headcell { padding: var(--space-1) var(--space-3); }
.fs-grid-headcell-text { color: var(--text-primary); font-size: var(--text-sm); font-weight: 700; }
.fs-grid-cell-current { border-color: var(--border-focus); }
.fs-grid-row-open .fs-grid-cell-text { font-weight: 600; }
.fs-grid-cells-hold .fs-grid-cell { padding: var(--space-2); }
.fs-grid-cell-open { border-color: var(--primary); }
.fs-grid-rowhead { background: var(--surface-elevated); }
.fs-grid-rowhead .fs-grid-cell-text { font-weight: 700; }
.fs-grid-mark { color: var(--text-secondary); font-size: var(--text-sm); }
";
