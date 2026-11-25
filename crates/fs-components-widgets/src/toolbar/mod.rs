//! `toolbar`: a row of controls the keyboard walks along with the arrow keys; the whole row is ONE tab stop.
//!
//! Pattern: State (`model.rs`, pure: the items, the cursor, the toggles) + Observer (`view.rs`: `Live` draws the items again) +
//! Adapter. WAI-ARIA APG, toolbar: role `toolbar` with `aria-orientation` and a name (`aria-label`), Left and Right (Up and Down in a
//! vertical bar) move between the controls, Home and End jump to the first and the last, Enter or Space presses the control the
//! cursor is on; a toggle carries `aria-pressed`; a separator is `separator`. The move does not wrap (APG: optional); a disabled
//! control is skipped by the keys, unless it has a reason (`disabled_because`): then the cursor reaches it (APG: a locked control stays reachable so its reason can be read; the tooltip is on the item, which has no tab stop of its own) and Enter does nothing. Limits: one tab stop like `segmented` — the keyboard stays on the bar and the cursor is the
//! `fs-tool-current` mark; the head bar (`head-bar`) keeps one tab stop per button.

mod model;
mod styles;
mod view;

pub use model::{Item, Kind, Outcome, State};
pub use styles::STYLES;
pub use view::Toolbar;
