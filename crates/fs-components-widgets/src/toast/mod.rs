//! `toast`: a short message that comes, stays a while and goes by itself, or has an action ("Undo").
//!
//! Pattern: Facade + Observer (`handle.rs`: [`Toasts`]), State with an injected clock (`model.rs`), Adapter (`view.rs`). Kinds:
//! info, ok, warning, error (`Severity`); a warning and an error (role `alert`) stay until they are closed (the default of [`Times`]). The stack shows the newest [`MAX_VISIBLE`] toasts
//! (tunable) and counts the older ones. A pointer over a toast or the keyboard on its close mark holds its time (WCAG 2.2.1); how
//! long each kind stays is a setting ([`Times`]). WAI-ARIA APG alert/status: each toast has role `status` (info, ok) or `alert`
//! (warning, error) with the matching `aria-live` and `aria-atomic` (`Severity::live`). Role and live attributes are SET and can be read
//! by the host; a screen reader hears them only when Blinc passes them on (it maps `role` through a table of 12 roles and loses
//! dialog, alertdialog, status, alert and `aria-live`: upstream issue project-blinc/Blinc#79). Limits: no animation (a toast appears and goes at once).

mod handle;
mod labels;
mod model;
mod styles;
mod view;

pub use handle::{Toasts, now_ms};
pub use labels::{Labels, TEXTS};
pub use model::{Act, MAX_VISIBLE, Times, Toast};
pub use styles::STYLES;
