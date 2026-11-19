//! The views: one function per mode, written once for every basic type.
//!
//! Pattern: none — declares the modules and re-exports.

mod cell;
mod choice;
mod compact;
mod display;
mod input;
mod keywords;
mod keywords_words;
mod look;
mod reference;
mod say;
mod suggestions;
mod surface;

pub use cell::cell;
pub use choice::{ChoiceInput, SEGMENTS_UP_TO};
pub use compact::compact;
pub use display::display;
pub use input::FieldInput;
pub use keywords::KeywordsInput;
pub use look::Look;
pub use reference::{Candidate, ReferenceInput, ReferenceSource};
pub use say::{KeysAsText, Say};
