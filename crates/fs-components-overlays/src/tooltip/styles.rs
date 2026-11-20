//! The style sheet of the tooltip.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the tooltip: only theme variables.
pub const STYLES: &str = r"
.fs-tooltip-host { display: flex; flex-direction: column; align-items: flex-start; }
.fs-tooltip-slot { display: flex; flex-direction: column; }
.fs-tooltip { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--text-primary); box-shadow: var(--dialog-shadow); }
.fs-tooltip-text { color: var(--background); font-size: var(--text-xs); }
";
