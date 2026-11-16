//! `dropdown`: a menu that hangs on a trigger and stays inside the window.
//!
//! Pattern: Builder (entries) + Strategy (`place`, a pure rule). Why not `blinc_cn`'s dropdown: Blinc decides the flip only by the room under the
//! trigger (96 px) and ignores the height of the panel, so a menu on a tile at the bottom of the window (100 to 200 px of room
//! below, a taller panel) opened below the edge and its last lines could not be pressed. Here the panel has a known height (`panel::height`), [`place`] puts it under the trigger, above it (flip), or
//! holds it in the window and shortens it (it then scrolls). Every menu of ours that hangs on an anchor takes this one.
//!
//! The overlay is pushed with no size hint on purpose: Blinc then places it exactly where [`place`] says and does not decide again.

mod builder;
mod panel;
mod place;

pub use builder::{Dropdown, dropdown};
pub use panel::{Action, Entry, ROW_HEIGHT, SEPARATOR_HEIGHT, height};
pub use place::{Align, GAP, MARGIN, Placed, Rect, place};
