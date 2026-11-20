//! The text typed in the quick-add rows: one field per column, kept outside everything that is drawn again.
//!
//! Pattern: Registry — a field is made on first use and lives as long as the board handle, so a redraw of the board (any refresh) never
//! loses what the reader is typing; the text only goes when the host accepted the card.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use fs_components_primitives::{FieldSize, SearchField};

/// The fields of the quick-add rows, by column key. Clones share them.
#[derive(Clone, Default)]
pub struct Drafts {
    fields: Arc<Mutex<HashMap<String, SearchField>>>,
}

impl Drafts {
    /// The field of `column`; `hint` is its placeholder (the word of the host's language; empty keeps the one it has).
    pub fn field(&self, column: &str, hint: &str) -> SearchField {
        let mut map = self.fields.lock().unwrap_or_else(PoisonError::into_inner);
        let field = map
            .entry(column.to_owned())
            .or_insert_with(|| SearchField::new(hint).size(FieldSize::Body));
        if !hint.is_empty() {
            field.set_placeholder(hint);
        }
        field.clone()
    }

    /// What is typed in `column` (empty when nothing was).
    pub fn text(&self, column: &str) -> String {
        let map = self.fields.lock().unwrap_or_else(PoisonError::into_inner);
        map.get(column).map(SearchField::text).unwrap_or_default()
    }

    /// Put `text` in the field of `column` (the field is made when missing; a test or a host that pre-fills).
    pub fn set(&self, column: &str, text: &str) {
        self.field(column, "").set_text_at_end(text);
    }

    /// Give the keyboard to the field of `column` (on the next frame).
    pub fn focus(&self, column: &str) {
        self.field(column, "").focus();
    }

    /// Empty the field of `column`.
    pub fn clear(&self, column: &str) {
        let map = self.fields.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(field) = map.get(column) {
            field.set_text_at_end("");
        }
    }
}
