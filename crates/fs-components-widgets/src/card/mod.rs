//! `card`: the one component with a click and a carry (block `card` of the contract; Open UI Card).
//!
//! Pattern: none - this file only declares the parts and re-exports the public API. The parts: [`refs`] (what is carried), [`gesture`]
//! (the pointer machine), [`hit`] (the place under a point), [`targets`] (container and owner), [`keys`] (the table of the keys),
//! [`widget`], `variants` and `view` (the card), [`layer`] (ghost and line), [`measured`] (places from the laid-out tree).

pub mod gesture;
pub mod hit;
pub mod keys;
pub mod layer;
pub mod measured;
pub mod refs;
mod styles;
pub mod targets;
mod variants;
mod view;
pub mod widget;

pub use hit::{Line, Placed, Rect, Zone, line_at, target_at};
pub use keys::{Mode, Step};
pub use measured::{BlincBounds, Bounds, MeasuredTargets, Scrolled};
pub use refs::{CardRef, Carry, Target};
pub use styles::STYLES;
pub use targets::{ACTIVATE, CardEvents, DROP, DropTargets, MENU, NoEvents};
pub use widget::{Card, Outcome};
