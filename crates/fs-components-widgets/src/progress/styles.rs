//! The style sheet of the progress bar.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the progress bar: only theme variables. Unknown is told by an outlined, empty-looking bar and by the words beside it; done is told
/// by the green fill with an edge and by the text (not by colour alone). The track is the accent tint so that the accent fill is at
/// least 3:1 against it in every style (the border colour was not: 1.0:1 in Windows dark).
pub const STYLES: &str = r"
.fs-progress { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); height: var(--space-6); }
.fs-progress-track { display: flex; flex-direction: row; height: var(--space-2); border-radius: var(--radius-full); background: var(--accent-subtle); }
.fs-progress-fill { height: var(--space-2); border-radius: var(--radius-full); background: var(--accent); }
.fs-progress-unknown .fs-progress-fill { background: var(--accent-subtle); border-width: 2px; border-color: var(--accent); }
.fs-progress-done .fs-progress-fill { background: var(--success); border-width: 1px; border-color: var(--text-primary); }
.fs-progress-text { color: var(--text-primary); font-size: var(--text-sm); }
";
