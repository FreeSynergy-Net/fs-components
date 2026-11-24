//! `fs-components-core`: the core of the widgets, cut out of `fs-components-widgets` (split1, step B).
//!
//! Keys, live regions, focus, roving, icons, states, the text model, landmark, drafts, picture
//! check, soft wrap, source errors and the write port of a form. Everything else of the widgets builds on it;
//! `fs-components-widgets` re-exports it under the old paths.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod drafts;
pub mod focus;
pub mod form_read;
pub mod form_view;
pub mod form_write;
pub mod icon;
pub mod icon_export;
pub mod icon_kind;
pub mod icon_named;
pub mod icon_theme;
pub mod keys;
pub mod landmark;
pub mod live;
pub mod picture;
pub mod roving;
pub mod softwrap;
pub mod source_error;
pub mod states;
pub mod text;

/// What a widget returns to be placed in a layout: the one place a Blinc type is visible.
pub use blinc_layout::prelude::ElementBuilder as Element;
// probe: touch (never merged)
