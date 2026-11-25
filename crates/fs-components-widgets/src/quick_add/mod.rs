//! Quick add: what Enter, Shift+Enter and a pasted text come to, for anything that is created from typed lines (a card, a task, a list item).
//!
//! Pattern: none - a pure function and its result type (in `submit`). Modules and re-exports only here.

pub mod form_view;
mod submit;

pub use submit::{Quick, submit};
