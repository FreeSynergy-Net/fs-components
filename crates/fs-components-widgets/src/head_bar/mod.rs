//! `head-bar`: the bar on top of an area, with a button or two at each end and a title between them (the header of the
//! contacts list: plus, title, check, menu; the header of a person: pencil, dots).
//!
//! Pattern: Composite — a bar is three slots (left, title, right) filled with elements; [`head_button`] is the round symbol button
//! that goes into them. The bar knows no program: a press is a callback, and every button says in words what it does
//! (`aria-label`, title) because a symbol alone cannot be read aloud.

mod bar;

pub use bar::{HeadBar, Press, STYLES, head_button, head_face};
