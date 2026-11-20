//! `progress`: how far a task has come, or that it has begun and nobody knows how far.
//!
//! Pattern: Observer + Facade — a clonable handle over the value; `set` draws the bar again through `Live`. WAI-ARIA `progressbar`:
//! `aria-valuenow`, `aria-valuemin`, `aria-valuemax` and `aria-valuetext` ("45 %") while the end is known; while it is not
//! (`Progress::unknown`) those are left out, as ARIA asks, and the bar shows a moving-free striped fill (no animation: it would run
//! forever and cost frames). The bar is named by `aria-label` ("Copying"). Done (100 %) has its own class and the text says it. A bar is
//! read, not set: no keys, no tab stop. Limits: horizontal only.

mod styles;
mod view;

pub use styles::STYLES;
pub use view::Progress;
