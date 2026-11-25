//! The views: one function per mode, written once for every basic type.
//!
//! Pattern: none — declares the modules and re-exports.

mod area;
mod cell;
mod choice;
mod compact;
mod display;
mod entry;
mod input;
mod input_parts;
mod keywords;
mod keywords_words;
mod link;
mod look;
mod reference;
mod say;
mod shown;
mod suggestions;
mod surface;

pub use area::{area_styles, field_styles};
pub use cell::cell;
pub use choice::{ChoiceInput, SEGMENTS_UP_TO};
pub use compact::{compact, compact_text, compact_width, compact_with, text_width};
pub use display::{display, display_with};
pub use input::FieldInput;
pub use keywords::KeywordsInput;
pub use link::{Follow, LINK_CONTRAST, link_ink};
pub use look::Look;
pub use reference::{Candidate, ReferenceInput, ReferenceSource};
pub use say::{KeysAsText, Say};
