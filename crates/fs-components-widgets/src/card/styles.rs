//! The style sheet of the card.
//!
//! Pattern: none - only theme variables (constants, no behaviour).

/// CSS of the card: only theme variables. Every state differs by more than colour (border width, opacity, the reason as text), so
/// it is told apart in the high-contrast themes too.
pub const STYLES: &str = r"
.fs-card { display: flex; flex-direction: column; padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface-elevated); }
.fs-card:hover { border-color: var(--primary); background: var(--accent-subtle); }
.fs-card-hover { border-color: var(--primary); background: var(--accent-subtle); }
.fs-card:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-card-panel { padding: var(--space-4); background: var(--surface); }
.fs-card-header { display: flex; flex-direction: row; align-items: center; }
.fs-card-title { color: var(--text-primary); font-size: var(--text-sm); font-weight: 700; }
.fs-card-body { color: var(--text-primary); font-size: var(--text-sm); }
.fs-card-footer { color: var(--text-secondary); font-size: var(--text-xs); margin-top: var(--space-1); }
.fs-card-held { border-width: 3px; border-color: var(--primary); background: var(--accent-subtle); }
.fs-card-carried { opacity: 0.4; border-width: 2px; border-color: var(--primary); }
.fs-card-locked { opacity: 0.7; background: var(--surface); border-color: var(--text-secondary); }
.fs-card-drop-line { height: 3px; border-radius: 2px; background: var(--primary); }
.fs-card-drop-blocked { background: var(--error); }
.fs-card-ghost { display: flex; flex-direction: column; padding: var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: var(--primary); background: var(--surface-elevated); opacity: 0.85; }
.fs-card-ghost-blocked { border-color: var(--error); }
.fs-card-ghost-reason { margin-top: var(--space-1); color: var(--error); font-size: var(--text-sm); font-weight: 700; }
";
