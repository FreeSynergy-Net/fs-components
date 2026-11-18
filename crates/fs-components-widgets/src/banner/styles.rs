//! The style sheet of the banner.
//!
//! Pattern: none — only theme variables (constants, no behaviour). The ground of a step is the one of `alert`.

/// CSS of the banner: only theme variables.
pub const STYLES: &str = r"
.fs-banner { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); width: 100%; padding: var(--message-pad) var(--space-3); border-width: 1px; border-color: var(--border); background: var(--surface); }
.fs-banner-info { border-left-color: var(--info); border-left-width: var(--toast-edge); background: var(--alert-info-bg); }
.fs-banner-ok { border-left-color: var(--success); border-left-width: var(--toast-edge); background: var(--alert-ok-bg); }
.fs-banner-warning { border-left-color: var(--warning); border-left-width: var(--toast-edge); background: var(--alert-warning-bg); }
.fs-banner-error { border-left-color: var(--error); border-left-width: var(--toast-edge); background: var(--alert-error-bg); }
.fs-banner-icon { display: flex; align-items: center; flex-shrink: 0; }
.fs-banner-text { flex-grow: 1; color: var(--text-primary); font-size: var(--text-sm); }
.fs-banner-close { display: flex; align-items: center; justify-content: center; flex-shrink: 0; width: 24px; height: 24px; border-radius: var(--radius-default); cursor: pointer; }
.fs-banner-close:hover { background: var(--accent-subtle); }
.fs-banner-close:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 1px; }
";
