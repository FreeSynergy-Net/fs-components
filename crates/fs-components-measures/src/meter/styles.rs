//! The style sheet of the meter.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the meter: only theme variables. The zone is told by the word beside the bar as well as the colour of the fill; the fill has an edge in the text colour, because the
/// zone colours are not 3:1 against any ground in every style (green 2.0:1 in Apple), and the length of the fill is the value.
pub const STYLES: &str = r"
.fs-meter { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); height: var(--space-6); }
.fs-meter-track { display: flex; flex-direction: row; height: var(--space-2); border-radius: var(--radius-full); background: var(--accent-subtle); }
.fs-meter-fill { height: var(--space-2); border-radius: var(--radius-full); border-width: 1px; border-color: var(--text-primary); }
.fs-meter-good .fs-meter-fill { background: var(--success); }
.fs-meter-fair .fs-meter-fill { background: var(--warning); }
.fs-meter-poor .fs-meter-fill { background: var(--error); }
.fs-meter-text { color: var(--text-primary); font-size: var(--text-sm); }
";
