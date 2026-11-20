//! `tree`: a hierarchy of items that open and close, with the roles and the keys of the APG tree view; the whole tree is ONE tab stop.
//!
//! Pattern: State (`model.rs`, pure: the nodes in an arena, which are open, the cursor, the pick) + Observer (`view.rs`: `Live` draws
//! the items again) + Adapter. WAI-ARIA APG, tree view: role `tree` (named by `aria-label`), items `treeitem` with `aria-level`,
//! `aria-setsize`, `aria-posinset`, `aria-selected`, parents with `aria-expanded`, their children in a `group`. Down and Up walk the
//! visible items, Right opens a closed parent and goes into an open one (its first child), Left closes an open parent and from a
//! leaf (or a closed parent) goes to the parent, Home and End jump, Enter or Space picks the item the cursor is on. Limits: the tree is
//! ONE tab stop and the cursor is the `fs-treeitem-current` mark (like `segmented`); no type-ahead, no `*` (open all siblings), no multi
//! select. `tree_view` is the older adapter over Blinc's own tree (no roles); this one is the one with the APG.

mod model;
mod styles;
mod view;

pub use model::{Outcome, State};
pub use styles::STYLES;
pub use view::Tree;
