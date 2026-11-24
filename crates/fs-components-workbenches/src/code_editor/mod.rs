//! The code editor as a [`FormView`](fs_components_core::form_view::FormView) of the form `single-record@1.1.0` (widget `CodeEditor`,
//! block `code-editor`): one text with line numbers, colours by its media type, an overview map and, for Markdown, a preview.
//!
//! Pattern: Abstract Factory (`View`) producing a handle (`Mounted`) + Memento (`Draft`: the text as the module said it, to go
//! back to) — the draft is the only thing typed into; the module keeps the truth and hears of a change only by `edit`.

mod draft;
mod draw;
mod form_view;
mod wiring;

#[cfg(test)]
mod tests;

pub use form_view::{View, WIDGET};
