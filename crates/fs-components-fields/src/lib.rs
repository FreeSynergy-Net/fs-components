//! `fs-components-fields`: the components of the basic types (ADR 0066, item 4): input, display,
//! compact and cell, for e-mail address, name, phone number, postal address, date, link and many-line text, in any
//! style of `fs-components-styles`.
//!
//! The rules and the structure of a type are pure ([`BasicField`], `kinds/`); the views in
//! `view/` are written once and read a [`Look`] (a style and light or dark). A program never
//! draws a field: it names a [`Kind`], a [`Mode`] and a style.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Colours are written as hex `0xRRGGBB`; `over` narrows a mix of two bytes back to one byte.
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation
)]

pub mod controls;
pub mod field;
pub mod kinds;
pub mod limit;
pub mod mode;
pub mod problem;
pub mod slot;
pub mod suggest;
pub mod view;

pub use field::{BasicField, Entry, Segment};
pub use kinds::Kind;
pub use limit::{Limit, LimitSetting};
pub use mode::Mode;
pub use problem::Problem;
pub use slot::{FieldSlot, SlotReason, SlotState};
pub use suggest::{Suggest, SuggestFromList};
pub use view::{
    Candidate, ChoiceInput, FieldInput, Follow, KeysAsText, KeywordsInput, LINK_CONTRAST, Look,
    ReferenceInput, ReferenceSource, SEGMENTS_UP_TO, Say, area_styles, cell, compact, compact_with,
    display, display_with, link_ink,
};
