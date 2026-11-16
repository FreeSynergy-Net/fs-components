//! The style sheet of the controls.
//!
//! Pattern: none — only theme variables (constants, no behaviour). `--switch-width`, `--switch-height` and `--segment-height` are
//! tokens of the style (`fs-components-styles`, `styles/block_tokens.rs`), every style sets them.

/// CSS of the controls: only theme variables. The state is told by position and by colour.
pub const STYLES: &str = r"
.fs-switch-hit { display: flex; flex-direction: row; border-radius: var(--radius-default); }
.fs-switch-hit:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-switch { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); cursor: pointer; }
.fs-switch-track { display: flex; flex-direction: row; align-items: center; justify-content: flex-start; flex-shrink: 0; width: var(--switch-width); height: var(--switch-height); padding: 2px; border-radius: var(--radius-full); border-width: 1px; border-color: var(--control-border); background: var(--input-bg); }
.fs-switch-thumb { width: calc(var(--switch-height) - 8px); height: calc(var(--switch-height) - 8px); border-radius: var(--radius-full); }
.fs-switch-off .fs-switch-thumb { background: var(--text-secondary); }
.fs-switch-on .fs-switch-track { justify-content: flex-end; border-color: var(--accent); background: var(--accent); }
.fs-switch-on .fs-switch-thumb { background: var(--text-inverse); }
.fs-switch-mixed .fs-switch-track { justify-content: center; border-color: var(--accent); background: var(--input-bg); }
.fs-switch-mixed .fs-switch-thumb { background: var(--accent); }
.fs-switch-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-switch-disabled { opacity: 0.5; cursor: default; }
.fs-seg { display: flex; flex-direction: row; align-self: flex-start; height: var(--segment-height); border-width: 1px; border-color: var(--control-border); border-radius: var(--radius-default); overflow: hidden; }
.fs-seg:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-seg-options { display: flex; flex-direction: row; height: 100%; gap: 1px; background: var(--control-border); }
.fs-seg-item { display: flex; flex-direction: row; align-items: center; justify-content: center; padding: 0 var(--space-3); background: var(--surface); cursor: pointer; }
.fs-seg-item:hover { background: var(--secondary-hover); }
.fs-seg-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-seg-on { background: var(--accent); }
.fs-seg-on:hover { background: var(--accent); }
.fs-seg-on .fs-seg-label { color: var(--text-inverse); }
.fs-seg-off { cursor: default; }
.fs-seg-off .fs-seg-label { color: var(--text-tertiary); }
.fs-seg-disabled .fs-seg-label { color: var(--text-tertiary); }
";
