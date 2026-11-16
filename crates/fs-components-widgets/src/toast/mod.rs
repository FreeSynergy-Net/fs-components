//! `toast`: a short message that comes, stays a while and goes by itself, or has an action ("Undo").
//!
//! Pattern: Facade + Observer (`handle.rs`: [`Toasts`]), State with an injected clock (`model.rs`), Adapter (`view.rs`). Kinds:
//! info, ok, warning, error (`Severity`); an error stays until it is closed. The stack shows the newest [`MAX_VISIBLE`] toasts
//! (tunable) and counts the older ones. A pointer over a toast holds its time. Limits: no animation (a toast appears and goes
//! at once), and a toast is not announced by a screen reader here (Blinc has no live region; the `role` is set for the day it has).

mod handle;
mod labels;
mod model;
mod styles;
mod view;

pub use handle::{Toasts, now_ms};
pub use labels::{Labels, TEXTS};
pub use model::{Act, MAX_VISIBLE, Toast};
pub use styles::STYLES;
