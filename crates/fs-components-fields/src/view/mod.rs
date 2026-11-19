//! The views: one function per mode, written once for every basic type.
//!
//! Pattern: none — declares the modules and re-exports.

mod area;
mod cell;
mod compact;
mod display;
mod input;
mod link;
mod look;
mod say;
mod suggestions;
mod surface;

pub use area::area_styles;
pub use cell::cell;
pub use compact::{compact, compact_with};
pub use display::{display, display_with};
pub use input::FieldInput;
pub use link::{Follow, LINK_CONTRAST, link_ink};
pub use look::Look;
pub use say::{KeysAsText, Say};
