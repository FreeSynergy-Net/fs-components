//! `tabs`: a row of tabs, one of them chosen, with the panel that belongs to the chosen one.
//!
//! Pattern: State (`model.rs`, pure: the tabs, the chosen one, the cursor) + Observer (`view.rs`: `Live` draws the row and the panel
//! again) + Adapter. WAI-ARIA APG, tabs: the row is `tablist` (with `aria-orientation`), a tab is `tab` with `aria-selected` and
//! `aria-controls` naming its panel, the panel is `tabpanel` named by its tab (`aria-labelledby`). Left and Right (Up and Down in a
//! vertical row) move, Home and End jump, the move wraps. Two ways to choose, as in the APG: automatic (the choice follows the
//! cursor, the default) and manual (the cursor moves, Enter or Space chooses; `Tabs::manual`). A disabled tab is skipped by the keys.
//! Limits: the row is ONE tab stop (like `segmented`): the keyboard stays on the row and the cursor is the `fs-tab-current` mark,
//! not a moving focus; no tab can be closed or moved here.

pub mod form_view;
mod model;
mod styles;
mod view;

pub use model::{Outcome, State, Tab};
pub use styles::STYLES;
pub use view::Tabs;
