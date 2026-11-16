//! The style sheet of the toasts.
//!
//! Pattern: none — only theme variables (constants, no behaviour). `--dialog-shadow` is a token of the style.

/// CSS of the toasts: only theme variables. The kind tints the ground and draws the edge; the glyph tells the kind apart too.
pub const STYLES: &str = r"
.fs-toasts { display: flex; flex-direction: column; align-items: flex-end; gap: var(--space-2); }
.fs-toasts-more { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-toast { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); width: 100%; max-width: var(--toast-width); padding: var(--message-pad) var(--space-3); border-radius: var(--message-radius); border-width: 1px; border-color: var(--border); border-left-width: var(--toast-edge); background: var(--surface); box-shadow: var(--dialog-shadow); }
.fs-toast-info { border-left-color: var(--info); background: color-mix(in srgb, var(--info) var(--message-tint), var(--surface)); }
.fs-toast-ok { border-left-color: var(--success); background: color-mix(in srgb, var(--success) var(--message-tint), var(--surface)); }
.fs-toast-warning { border-left-color: var(--warning); background: color-mix(in srgb, var(--warning) var(--message-tint), var(--surface)); }
.fs-toast-error { border-left-color: var(--error); background: color-mix(in srgb, var(--error) var(--message-tint), var(--surface)); }
.fs-toast-icon { display: flex; align-items: center; flex-shrink: 0; }
.fs-toast-text { flex-grow: 1; color: var(--text-primary); font-size: var(--text-sm); }
.fs-toast-close { display: flex; align-items: center; justify-content: center; flex-shrink: 0; width: 24px; height: 24px; border-radius: var(--radius-default); cursor: pointer; }
.fs-toast-close:hover { background: var(--accent-subtle); }
.fs-toast-close:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 1px; }
";
