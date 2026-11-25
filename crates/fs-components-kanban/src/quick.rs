//! The quick entry at the foot of a column: what Enter, Shift+Enter and a pasted text come to, as cards.
//!
//! Pattern: Adapter - the rule (one entry per non-empty line, ask first when more than one, reversed at the top) is
//! `fs_components_widgets::quick_add::submit`, one module for everything that is made from typed lines; here only each title becomes a
//! [`NewCard`] of the column.

use crate::requests::NewCard;
use fs_components_widgets::quick_add;

/// What a submit comes to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Quick {
    /// Nothing to create (no letter in it).
    Nothing,
    /// Create these now.
    Create(Vec<NewCard>),
    /// More than one line: ask first, then create these.
    Confirm(Vec<NewCard>),
}

/// The cards `text` stands for in `column`; `top` = Shift+Enter. The order is that of `quick_add::submit` (the order to ask the program).
pub fn submit(column: &str, text: &str, top: bool) -> Quick {
    let cards = |titles: Vec<String>| {
        titles
            .into_iter()
            .map(|title| NewCard {
                column: column.to_owned(),
                title,
                top,
            })
            .collect()
    };
    match quick_add::submit(text, top) {
        quick_add::Quick::Nothing => Quick::Nothing,
        quick_add::Quick::Create(t) => Quick::Create(cards(t)),
        quick_add::Quick::Confirm(t) => Quick::Confirm(cards(t)),
    }
}
