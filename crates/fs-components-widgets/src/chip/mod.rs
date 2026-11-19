//! `chip`: a row of small removable labels (the keywords of a text, the people of a share); the row is ONE tab stop.
//!
//! Pattern: State (`model.rs`, pure: the chips, the cursor, what a key does) + Observer (`view.rs`: `Live` draws the chips again, the
//! row's own element stays so the keyboard stays) + Adapter. The row is a `list` named by `aria-label`, a chip a `listitem` with a
//! remove button named "Remove <chip>" (MF2 de/en). Left/Right move the cursor (no wrap), Home/End jump, Delete or Backspace removes
//! the chip under the cursor and the cursor stays at the same place (or the last). Limits: no chip INPUT (typing a new chip is the
//! field `keywords` of the field bundles); the remove button is for the pointer, the keyboard uses Delete; `badge` (a count or a
//! dot) stays in `badge.rs`.

mod labels;
mod model;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use model::{Chip, Outcome, State};
pub use styles::STYLES;
pub use view::ChipList;
