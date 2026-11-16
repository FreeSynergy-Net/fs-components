//! `dropdown`: a menu that hangs on a trigger and stays inside the window.
//!
//! Pattern: Builder (entries) + Strategy (`place`, a pure rule). Why not `blinc_cn`'s dropdown: it opens under its trigger and
//! flips only when under 96 px are left, with a guess for the height of the panel that is far too small (about 26 px a line
//! against 38 real), so a menu on a tile at the bottom of the window opened below the edge and its last lines could not be
//! pressed. Here the panel has a known height (`panel::height`), [`place`] puts it under the trigger, above it (flip), or
//! holds it in the window and shortens it (it then scrolls). Every menu of ours that hangs on an anchor takes this one.
//!
//! The overlay is pushed with no size hint on purpose: Blinc then places it exactly where [`place`] says and does not decide again.

mod builder;
mod panel;
mod place;

pub use builder::{Dropdown, dropdown};
pub use panel::{Action, Entry, ROW_HEIGHT, SEPARATOR_HEIGHT, height};
pub use place::{Align, MARGIN, Placed, Rect, place};
