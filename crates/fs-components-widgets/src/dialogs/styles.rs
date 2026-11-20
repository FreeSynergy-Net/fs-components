//! The style sheet of the dialogs.
//!
//! Pattern: none — only theme variables (constants, no behaviour). `--scrim` and `--dialog-shadow` are the tokens of the style
//! (`fs-components-styles`, `styles/block_tokens.rs`), every style sets them.

/// CSS of the dialogs: only theme variables.
pub const STYLES: &str = r"
.fs-dlg-backdrop { display: flex; flex-direction: row; align-items: center; justify-content: center; background: var(--scrim); }
.fs-dlg-slot { display: flex; flex-direction: column; width: 100%; max-width: var(--dialog-width); }
.fs-dlg { display: flex; flex-direction: column; align-self: flex-start; width: 100%; gap: var(--space-3); padding: var(--dialog-pad); border-radius: var(--dialog-radius); background: var(--surface); border-width: 1px; border-color: var(--border); box-shadow: var(--dialog-shadow); }
.fs-dlg-confirm, .fs-dlg-choice, .fs-dlg-input, .fs-dlg-license, .fs-dlg-alert { border-color: var(--border); }
.fs-dlg-danger { border-color: var(--dialog-danger-edge); }
.cn-button--destructive { background: var(--danger-bg); color: var(--danger-text); border-width: var(--danger-line-width); border-color: var(--danger-line); }
.cn-button--destructive:hover:not(:disabled) { background: var(--danger-bg-hover); }
.cn-button--destructive:active:not(:disabled) { background: var(--danger-bg-active); }
.fs-dlg-head { display: flex; flex-direction: column; width: 100%; }
.fs-dlg-danger .fs-dlg-head { padding-bottom: var(--dialog-head-gap); border-bottom-width: var(--dialog-head-line); border-bottom-color: var(--border-error); }
.fs-dlg-busy { opacity: 0.7; }
.fs-dlg-title { color: var(--text-primary); font-size: var(--text-lg); font-weight: 600; }
.fs-dlg-body { display: flex; flex-direction: column; width: 100%; gap: var(--space-2); }
.fs-dlg-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-dlg-footer { display: flex; flex-direction: row; justify-content: flex-end; gap: var(--space-2); }
.fs-ch { display: flex; flex-direction: column; width: 100%; gap: var(--space-2); }
.fs-ch-list { display: flex; flex-direction: column; width: 100%; border-radius: var(--radius-default); }
.fs-ch-list:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-ch-options { display: flex; flex-direction: column; width: 100%; gap: var(--space-1); }
.fs-ch-opt { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border-secondary); cursor: pointer; }
.fs-ch-opt:hover { background: var(--accent-subtle); }
.fs-ch-opt-on { border-color: var(--accent); background: var(--accent-subtle); }
.fs-ch-opt-off { opacity: 0.5; cursor: default; }
.fs-ch-mark { display: flex; align-items: center; justify-content: center; flex-shrink: 0; width: 16px; height: 16px; border-radius: var(--radius-full); border-width: 2px; border-color: var(--control-border); }
.fs-ch-opt-on .fs-ch-mark { border-color: var(--accent); }
.fs-ch-dot { width: 8px; height: 8px; border-radius: var(--radius-full); background: var(--accent); }
.fs-ch-words { display: flex; flex-direction: column; }
.fs-ch-label { color: var(--text-primary); font-size: var(--text-sm); }
.fs-ch-hint { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-dlg-input-body { display: flex; flex-direction: column; width: 100%; gap: var(--space-2); }
.fs-dlg-input-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-dlg-input-field { display: flex; flex-direction: column; width: 100%; }
.fs-lic-name { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-lic-text { display: flex; flex-direction: column; width: 100%; border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--code-bg); }
.fs-lic-text:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-lic-lines { display: flex; flex-direction: column; padding: var(--space-3); }
.fs-lic-line { color: var(--text-primary); font-family: var(--font-mono); font-size: var(--text-xs); }
";
