//! `dropdown`: a menu that hangs on a trigger and stays inside the window.
//!
//! Pattern: Builder (entries) + Strategy (`place`, a pure rule). Why not `blinc_cn`'s dropdown: Blinc decides the flip only by the room under the
//! trigger (96 px) and ignores the height of the panel, so a menu on a tile at the bottom of the window (100 to 200 px of room
//! below, a taller panel) opened below the edge and its last lines could not be pressed. Here the panel has a known height (`panel::height`), [`place`] puts it under the trigger, above it (flip), or
//! holds it in the window and shortens it (it then scrolls). Every menu of ours that hangs on an anchor takes this one.
//!
//! M3: the lines can come from the command model ([`Dropdown::commands`], [`entries_from_model`]), the same resolved items as the title-bar menu
//! and the context menu, and a panel that opens at a point (a context menu) is placed by [`place_at_point`].
//!
//! The overlay is pushed with no size hint on purpose: Blinc then places it exactly where [`place`] says and does not decide again.

mod builder;
mod model;
mod panel;
mod place;

pub use builder::{Dropdown, dropdown};
pub use model::entries_from_model;
pub use panel::{Action, Entry, ROW_HEIGHT, SEPARATOR_HEIGHT, height};
pub use place::{Align, GAP, MARGIN, Placed, Rect, place, place_at_point};
