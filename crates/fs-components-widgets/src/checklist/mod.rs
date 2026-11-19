//! `checklist`: a list of things to tick off that the person can add to, remove from and put in order; the list is ONE tab stop.
//!
//! Pattern: State (`model.rs`, pure: the items, the cursor, what a key does) + Observer (`view.rs`: `Live` draws the items again, the
//! list's own element and the line to add stay, so the keyboard stays) + Adapter over Blinc's `text_input` for the new item. WAI-ARIA
//! APG, checkbox: the list is a `group` named by `aria-label`, an item a `checkbox` with `aria-checked`. Up and Down move the cursor
//! (no wrap), Home and End jump, Space or Enter ticks or unticks, Delete or Backspace removes, Alt+Up and Alt+Down move the item in the
//! list. A new item is typed into the line and made by Enter (a blank is no item). Limits: the keyboard moves the cursor, not the
//! focus (like `segmented`); the move of an item has keys only (no drag); an item is one line of text.

mod labels;
mod model;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use model::{Item, Outcome, State};
pub use styles::STYLES;
pub use view::Checklist;
