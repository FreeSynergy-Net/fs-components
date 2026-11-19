//! `carousel`: slides one at a time with previous and next buttons; no automatic rotation.
//!
//! Pattern: State (`model.rs`, pure: count, current, wrap, what a key does) + Observer (`view.rs`: `Live` draws the slide again; the
//! buttons and the root stay, so the keyboard stays) + Adapter. WAI-ARIA APG, carousel: the root is a `region` with
//! `aria-roledescription="carousel"` and a name, every slide a `group` with `aria-roledescription="slide"` and the name "n / m",
//! previous and next are buttons with names (MF2 de/en). Left/Right on the carousel go to the previous/next slide, Home/End to the
//! first/last; the slides wrap (a setting). Limits: NO automatic rotation (the APG then asks for a pause control and for stopping on
//! focus: a carousel that never moves needs neither); one slide at a time; no tab dots.

mod labels;
mod model;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use model::{Outcome, State};
pub use styles::STYLES;
pub use view::Carousel;
