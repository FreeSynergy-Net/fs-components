//! The quick entry at the foot of a column: what Enter, Shift+Enter and a pasted text come to.
//!
//! Pattern: none - pure functions. Empty lines are nothing; more than one line asks "N cards?" first (never a silent flood).

use crate::requests::NewCard;

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

/// The cards `text` stands for in `column`; `top` = Shift+Enter. One card per non-empty line, **in the order to ask the program**:
/// as typed for the end of the column; reversed for the top, so that each one lands above the one before and the first
/// line ends up first.
pub fn submit(column: &str, text: &str, top: bool) -> Quick {
    let mut cards: Vec<NewCard> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|title| NewCard {
            column: column.to_owned(),
            title: title.to_owned(),
            top,
        })
        .collect();
    if top {
        cards.reverse();
    }
    match cards.len() {
        0 => Quick::Nothing,
        1 => Quick::Create(cards),
        _ => Quick::Confirm(cards),
    }
}
