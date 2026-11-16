//! The style sheet of the state surfaces.
//!
//! Pattern: none — only theme variables (constants, no behaviour). `--skeleton-base` is a token of the style.

/// CSS of the state surfaces: only theme variables. The step tints the ground and draws the edge; the glyph tells the step apart too.
pub const STYLES: &str = r"
.fs-alert, .fs-partial { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); padding: var(--message-pad) var(--space-3); border-radius: var(--message-radius); border-width: 1px; border-color: var(--border); border-left-width: var(--toast-edge); background: var(--surface); }
.fs-alert-info { border-left-color: var(--info); background: var(--alert-info-bg); }
.fs-alert-ok { border-left-color: var(--success); background: var(--alert-ok-bg); }
.fs-alert-warning, .fs-partial { border-left-color: var(--warning); background: var(--alert-warning-bg); }
.fs-alert-error { border-left-color: var(--error); background: var(--alert-error-bg); }
.fs-alert-icon, .fs-partial-icon { display: flex; align-items: center; flex-shrink: 0; }
.fs-alert-words, .fs-partial-words { display: flex; flex-direction: column; flex-grow: 1; gap: var(--space-1); }
.fs-alert-title { color: var(--text-primary); font-size: var(--text-sm); font-weight: 600; }
.fs-alert-text, .fs-partial-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-partial-list { display: flex; flex-direction: column; }
.fs-partial-item { color: var(--text-primary); font-size: var(--text-xs); }
.fs-empty { display: flex; flex-direction: column; align-items: center; gap: var(--space-2); padding: var(--space-6); }
.fs-empty-title { color: var(--text-primary); font-size: var(--text-lg); font-weight: 600; }
.fs-empty-text { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-skel { display: flex; flex-direction: column; gap: var(--space-3); width: 100%; }
.fs-skel-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); width: 100%; }
.fs-skel-bar { background: var(--skeleton-base); border-radius: var(--skeleton-radius); }
.fs-skel-line { height: var(--skeleton-height); }
.fs-skel-circle { border-radius: var(--radius-full); flex-shrink: 0; }
.fs-skel-w100 { width: 100%; }
.fs-skel-w70 { width: 70%; }
.fs-skel-w60 { width: 60%; }
";
