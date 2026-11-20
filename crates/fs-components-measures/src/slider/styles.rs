//! The style sheet of the slider (and of the range slider, which shares its parts).
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the slider: only theme variables. The filled part and the thumb carry the value (position, not colour alone); the thumb
/// that has the keyboard has a ring.
pub const STYLES: &str = r"
.fs-slider { display: flex; flex-direction: row; align-items: center; height: var(--space-8); }
.fs-slider-track { display: flex; flex-direction: column; }
.fs-slider-rail { height: var(--space-2); border-radius: var(--radius-full); background: var(--accent-subtle); }
.fs-slider-fill { height: var(--space-2); border-radius: var(--radius-full); background: var(--accent); }
.fs-slider-thumb { width: var(--space-5); height: var(--space-5); border-radius: var(--radius-full); border-width: 2px; border-color: var(--accent); background: var(--surface); cursor: pointer; }
.fs-slider-thumb:hover { background: var(--accent-subtle); }
.fs-slider-keys:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; border-radius: var(--radius-full); }
.fs-slider-thumb-focus { border-color: var(--border-focus); }
.fs-slider-thumb-active { background: var(--accent-subtle); }
.fs-slider-value { color: var(--text-primary); font-size: var(--text-sm); }
";
