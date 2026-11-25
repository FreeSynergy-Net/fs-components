//! The style sheet of the board workbench.
//!
//! Pattern: none - only theme variables. Big column names, readable text, a quiet card; the colour that carries meaning
//! (over the limit, refused) always comes with a symbol or a sentence.

/// The style sheet of the board and of the widgets it uses; add it to the theme of the window that holds a
/// [`BoardBench`](crate::BoardBench).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-board-root { display: flex; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-board-title-row { flex-shrink: 0; padding: var(--space-4) var(--space-5) var(--space-2) var(--space-5); }
.fs-board-title { font-size: 28px; font-weight: 700; }
.fs-board-col-name { font-size: 18px; font-weight: 700; }
.fs-board-wip { margin-left: var(--space-3); gap: var(--space-1); color: var(--text-secondary); font-size: 14px; }
.fs-board-wip-over { padding: 2px var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); }
.fs-board-wip-symbol { font-size: 11px; }
.fs-board-hidden { margin-left: var(--space-3); color: var(--text-secondary); font-size: 13px; }
.fs-board-head-menu { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); color: var(--text-secondary); font-size: 18px; }
.fs-board-head-menu:hover { background: var(--accent-subtle); }
.fs-board-column { padding: 0; border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface); }
.fs-board-head { padding: 0 var(--space-2); border-radius: var(--radius-default); background: var(--accent-subtle); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-board-cell { min-height: 72px; border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--background); }
.fs-board-card { padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface-elevated); }
.fs-board-card:hover { border-color: var(--primary); }
.fs-board-card-title { font-size: 15px; font-weight: 600; }
.fs-board-card-line { margin-top: var(--space-1); }
.fs-board-card-meta { color: var(--text-secondary); font-size: 13px; }
.fs-board-card-more { padding-left: var(--space-2); }
.fs-board-card-cursor { border-width: 2px; border-color: var(--primary); }
.fs-board-card-held { border-width: 2px; border-color: var(--primary); background: var(--accent-subtle); }
.fs-board-card-carried { opacity: 0.45; }
.fs-board-card-locked { opacity: 0.75; }
.fs-board-lanes-missing { flex-shrink: 0; padding: var(--space-2) var(--space-5); color: var(--text-secondary); font-size: 14px; background: var(--warning-bg); }
.fs-board-lane { padding-bottom: var(--space-3); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface); }
.fs-board-lane .fs-disc-head { background: var(--accent-subtle); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-board-lane-columns { padding-top: var(--space-2); }
.fs-board-empty { padding: var(--space-3); color: var(--text-secondary); font-size: 14px; }
.fs-board-foot { padding: 0 var(--space-1); }
.fs-board-foot .fs-plan-entry-field { border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface-elevated); }
.fs-board-add { border-radius: var(--radius-default); color: var(--text-secondary); }
.fs-board-add:hover { background: var(--accent-subtle); color: var(--text-primary); }
.fs-board-add-text { font-size: 14px; padding: 0 var(--space-2); }
.fs-board-progress { margin-top: var(--space-1); border-radius: 2px; background: var(--border); }
.fs-board-progress-fill { border-radius: 2px; background: var(--primary); }
.fs-board-drop-line { height: 3px; border-radius: 2px; background: var(--primary); }
.fs-board-drop-blocked { background: var(--error); }
.fs-board-ghost { padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--primary); background: var(--surface-elevated); opacity: 0.85; }
.fs-board-ghost-blocked { border-color: var(--error); }
.fs-board-ghost-reason { margin-top: var(--space-1); color: var(--error); font-size: 13px; }
.fs-board-line { flex-shrink: 0; gap: var(--space-3); padding: var(--space-3) var(--space-5); border-top-width: 1px; border-top-color: var(--border); background: var(--surface); }
.fs-board-line-text { font-size: 15px; }
.fs-board-line-refused { background: var(--warning-bg); }
.fs-board-detail { padding: var(--space-4); gap: var(--space-3); border-left-width: 1px; border-left-color: var(--border); background: var(--surface); }
.fs-board-detail-top { gap: var(--space-2); }
.fs-board-detail-path { color: var(--text-secondary); font-size: 14px; }
.fs-board-detail-close { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); color: var(--text-secondary); }
.fs-board-detail-close:hover { background: var(--accent-subtle); }
.fs-board-detail-title { font-size: 22px; font-weight: 700; }
.fs-board-detail-prop { gap: var(--space-3); padding: var(--space-1) 0; }
.fs-board-detail-name { width: 110px; flex-shrink: 0; color: var(--text-secondary); font-size: 14px; }
.fs-board-detail-value { font-size: 15px; }
.fs-board-detail-unset { color: var(--text-secondary); font-size: 15px; }
.fs-board-detail-reason { color: var(--text-secondary); font-size: 15px; font-style: italic; }
.fs-board-detail-actions { margin-top: var(--space-3); }
";
