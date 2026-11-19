//! The style sheet of the carousel.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the carousel: only theme variables. The position "n / m" is shown in words as well as by the slide.
pub const STYLES: &str = r"
.fs-carousel { display: flex; flex-direction: column; gap: var(--space-2); }
.fs-carousel:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-carousel-bar { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); }
.fs-carousel-button { padding: var(--space-1) var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); cursor: pointer; }
.fs-carousel-button:hover { background: var(--accent-subtle); }
.fs-carousel-button:focus { border-color: var(--border-focus); outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-carousel-button-text { color: var(--text-primary); font-size: var(--text-sm); }
.fs-carousel-position { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-carousel-slide { padding: var(--space-3); border-radius: var(--radius-default); border-width: 1px; border-color: var(--border); background: var(--surface); }
";
