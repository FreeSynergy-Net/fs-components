//! The style sheet of the people workbench.
//!
//! Pattern: none — only theme variables; widget sheets are joined here.

/// The style sheet of the workbench and the widgets it uses; add it to the theme of the window that
/// holds a [`PeopleWidget`](crate::people::PeopleWidget).
pub fn styles() -> String {
    [LAYOUT, &fs_components_widgets::styles()].concat()
}

const LAYOUT: &str = r"
.fs-people-root { display: flex; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-people-side { flex-shrink: 0; border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-people-search { flex-shrink: 0; align-items: center; padding: var(--space-2) var(--space-3); }
.fs-people-search-field { flex-grow: 1; min-width: 0; width: 100%; }
.fs-people-section-head { padding: var(--space-3) var(--space-4) var(--space-2) var(--space-4); color: var(--text-secondary); font-size: var(--text-base); font-weight: 700; }
.fs-people-rows { padding-bottom: var(--space-3); }
.fs-contact-row { gap: var(--space-3); height: var(--space-14); padding: 0 var(--space-3); margin: 0 var(--space-2); border-radius: var(--radius-lg); cursor: pointer; }
.fs-contact-row:hover { background: var(--accent-subtle); }
.fs-contact-row:focus { background: var(--accent-subtle); }
.fs-contact-row-selected { background: var(--accent-subtle); }
.fs-contact-row-marked { background: var(--accent-subtle); }
.fs-contact-face { flex-shrink: 0; }
.fs-contact-name { min-width: 0; font-size: var(--text-lg); color: var(--text-primary); }
.fs-contact-check { flex-shrink: 0; width: var(--space-6); height: var(--space-6); border-radius: var(--radius-lg); border-width: 2px; border-color: var(--text-secondary); }
.fs-contact-check-on { background: var(--accent); border-color: var(--accent); }
.fs-people-empty { gap: var(--space-3); padding: var(--space-8) var(--space-4); color: var(--text-secondary); }
.fs-people-nodetails { color: var(--text-secondary); padding: var(--space-4); }
.fs-people-head-bar { border-bottom-width: 0px; }
.fs-people-cards { max-width: 520px; gap: var(--space-4); }
.fs-people-more { flex-shrink: 0; display: flex; flex-direction: row; align-items: center; justify-content: center; border-top-width: 1px; border-top-color: var(--border); }
.fs-people-more-button { margin: var(--space-2) var(--space-3); padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); background: var(--surface-elevated); cursor: pointer; }
.fs-people-more-button:hover { background: var(--accent-subtle); }
.fs-people-main { min-width: 0; }
.fs-people-bar { flex-shrink: 0; align-items: center; gap: var(--space-3); padding: var(--space-3); min-height: 60px; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-people-really { flex-shrink: 0; padding: var(--space-3); gap: var(--space-3); }
.fs-people-page { padding: var(--space-6) var(--space-4); gap: var(--space-5); }
.fs-people-spacer { flex-grow: 1; }
.fs-people-head { align-items: center; gap: var(--space-3); }
.fs-people-title-text { font-size: var(--text-2xl); font-weight: 700; text-align: center; }
.fs-people-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-people-section { gap: var(--space-2); }
.fs-people-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-people-field { gap: var(--space-3); }
.fs-people-label { width: 120px; flex-shrink: 0; color: var(--text-secondary); }
.fs-people-value { min-width: 0; }
.fs-people-notice { padding: var(--space-2) var(--space-3); background: var(--warning-bg); color: var(--text-primary); }
.fs-people-failed { padding: var(--space-4); background: var(--error-bg); border-radius: var(--radius-default); }
.fs-people-face { flex-shrink: 0; margin-right: var(--space-3); }
.fs-people-input { padding-bottom: var(--space-2); }
";
