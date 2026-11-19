//! `breadcrumb`: the way to where the person is, each step a thing to press.
//!
//! Pattern: Adapter over plain Blinc elements, with a pure plan ([`plan`]). The widget has
//! no words and no colours of its own and does not know what a step means. A step is a plain element
//! and a tab stop, so it may be drawn inside a live area and the window turns Enter and Space on it
//! into its click. The last step is where the person is: it is not a thing to press.
//!
//! WAI-ARIA APG, breadcrumb: the row is a `navigation` landmark with a name (`aria-label`, words in `labels.rs`), a step you can
//! press is a `link`, the last one carries `aria-current="page"`, the separators are not read (`aria-hidden`).

mod labels;
mod widget;

pub use labels::{Labels, TEXTS};
pub use widget::{Breadcrumb, Crumb, OnChoose, STYLES, plan};
