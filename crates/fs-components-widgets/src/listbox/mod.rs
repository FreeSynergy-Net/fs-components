//! `listbox`: a list of options the person picks from, one or several; the whole list is ONE tab stop.
//!
//! Pattern: State (`model.rs`, pure: the options, the cursor, the picked ones) + Observer (`view.rs`: `Live` draws the options again,
//! the list's own element stays so the keyboard stays) + Adapter. WAI-ARIA APG, listbox: role `listbox` (named by `aria-label`,
//! `aria-multiselectable` when several may be picked), every option is `option` with `aria-selected`. Up and Down move, Home and End
//! jump (no wrap). Single pick: the pick follows the cursor, as the APG's "selection follows focus". Several: Space (or Enter) toggles
//! the option under the cursor, Ctrl+A picks all. Disabled options are skipped. Limits: no type-ahead, no Shift range; the list is one
//! tab stop and the cursor is the `fs-option-current` mark (like `segmented`), not a moving focus.

mod model;
mod styles;
mod view;

pub use model::{Opt, Outcome, State};
pub use styles::STYLES;
pub use view::Listbox;
