//! `fs-components-rich`: the writing surface of the `RichTextEditor` building block, stage 1.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.
//!
//! A living surface on the typed block tree of `fs-components-editor` (ADR 0084: the editor is a view, the storage stays the
//! block model): paragraphs, headings 1 to 6, lists (numbered, bullets, tasks with a box one can click), code blocks, quotes
//! and rules, drawn in the tokens of the theme; shortcuts typed (`## `, `- `, `1. `, `> `, `[] `, ```` ``` ````, `---`) make the
//! blocks at once; a visible cursor walks over the blocks. Not in stage 1: tables, copy and paste, selection over blocks, the
//! slash menu, marks inside text (`**x**`).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod geometry;
mod look;
mod measure;
mod place;
mod styles;
mod surface;
mod view;

pub use geometry::{Caret, Geometry, Leaf, Line, Marker, Shape, Sort};
pub use look::Look;
pub use measure::Face;
pub use place::place;
pub use styles::STYLES;
pub use surface::{RichSurface, key_of_code};
