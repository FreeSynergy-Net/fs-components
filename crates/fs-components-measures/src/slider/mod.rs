//! `slider`: one value on a scale, set with the keys or the pointer; the thumb is the one tab stop.
//!
//! Pattern: State (`scale.rs`, pure: the scale and what a key does to a value) + Observer (`view.rs`: `Live` draws the slider again
//! after every change, the keyboard stays on the thumb by its id) + Adapter. WAI-ARIA APG, slider: the thumb has role `slider` with
//! `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-valuetext` (the caller's words for the value) and `aria-orientation`,
//! named by `aria-label`. Left, Down decrease and Right, Up increase by one step, Page Down and Page Up by a page (a setting), Home
//! and End go to the ends. The pointer sets the value where it presses on the track and follows a drag. Limits: horizontal only;
//! the thumb is drawn at a position computed from the width given to `build` (a slider in a layout that resizes is built again); the
//! keyboard sits on an invisible layer over the track (`fs-slider-keys`, role `slider`) and the drawn thumb (`<id>-thumb`) is the layer
//! that moves, because Blinc keeps the attributes of a live area whose sizes changed and loses the keyboard of one whose structure
//! changed;
//! no tick marks.

mod scale;
mod styles;
mod view;

pub use scale::{Scale, number};
pub use styles::STYLES;
pub(crate) use view::rail;
pub use view::{Slider, THUMB};
