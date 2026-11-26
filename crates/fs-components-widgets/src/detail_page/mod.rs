//! `detail-page`: the page of ONE record, read: a big round face, the name, a short text, what it is marked with, then the fields as
//! cards, several values of one field as several cards under ONE label.
//!
//! Pattern: Composite (page, sections, fields) + Adapter ([`form_view`]: the form `single-record` in, a [`Sheet`] out) + Abstract
//! Factory (`View`). The element knows no form and no module: it draws a [`Sheet`]; whoever has another source (the people
//! workbench) fills the same sheet. Nothing here changes a record.

pub mod form_view;
mod model;
mod view;

pub use model::{Field, FieldKind, Section, Sheet};
pub use view::{FACE, STYLES, page};
