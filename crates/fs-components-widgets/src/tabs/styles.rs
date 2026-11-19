//! The style sheet of the tabs.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the tabs: only theme variables. The chosen tab is told by weight and ground (not by colour alone); the cursor of the
/// manual way has its own edge; a disabled tab is weak and the pointer ignores it.
pub const STYLES: &str = r"
.fs-tabs { display: flex; flex-direction: row; gap: var(--space-1); padding: var(--space-1); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); }
.fs-tabs-vertical { flex-direction: column; }
.fs-tabs:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-tab { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: transparent; cursor: pointer; }
.fs-tab:hover { background: var(--accent-subtle); }
.fs-tab-label { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-tab-selected { background: var(--accent-subtle); }
.fs-tab-selected .fs-tab-label { color: var(--text-primary); font-weight: 600; }
.fs-tab-current { border-color: var(--border-focus); }
.fs-tab-disabled { cursor: default; }
.fs-tab-disabled .fs-tab-label { color: var(--text-tertiary); }
.fs-tabpanel { padding: var(--space-3); }
";
