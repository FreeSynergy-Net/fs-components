//! The style sheet of the popover.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the popover: only theme variables.
pub const STYLES: &str = r"
.fs-popover-host { display: flex; flex-direction: column; align-items: flex-start; }
.fs-popover { display: flex; flex-direction: column; gap: var(--space-2); padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); box-shadow: var(--dialog-shadow); }
.fs-popover-head { display: flex; flex-direction: row; justify-content: flex-end; }
.fs-popover-close { display: flex; align-items: center; justify-content: center; width: 24px; height: 24px; border-radius: var(--radius-default); cursor: pointer; }
.fs-popover-close:hover { background: var(--accent-subtle); }
.fs-popover-close:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 1px; }
";
