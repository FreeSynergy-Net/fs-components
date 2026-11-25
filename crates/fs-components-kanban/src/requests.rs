//! The plain data of the board workbench: what the program asks, what it answers, what the reader asked for.
//!
//! Pattern: Value objects. No program type appears here: a column key and a card key are opaque texts (a status, a stage,
//! a task id are the program's), and every sentence is already in the reader's language (the program's data).

/// Why something is not possible, a question first, what the program answers: the types of `fs-components-core` (`answer`), the same
/// for every component, re-exported here under the names the board always had.
pub use fs_components_widgets::answer::{Answer, Question, Reason};

/// A card to move: where from, where to, and after which visible card.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveRequest {
    /// The card.
    pub card: String,
    /// The column it leaves.
    pub from: String,
    /// The column it goes to.
    pub to: String,
    /// The place among the cards that are shown (counted without the card itself). A filter can hide cards, so the
    /// program should take `after`, not this number.
    pub index: usize,
    /// The shown card it comes right after; `None` = the top of the column.
    pub after: Option<String>,
    /// The reader said yes to the program's question (`Answer::Ask`).
    pub confirmed: bool,
}

impl MoveRequest {
    /// A request to move `card` from one column to another, after a shown card (not yet confirmed).
    pub fn new(
        card: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        index: usize,
        after: Option<String>,
    ) -> Self {
        Self {
            card: card.into(),
            from: from.into(),
            to: to.into(),
            index,
            after,
            confirmed: false,
        }
    }
}

/// Something the reader types at the foot of a column.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewCard {
    /// The column.
    pub column: String,
    /// The title (trimmed, never empty).
    pub title: String,
    /// At the top of the column (Shift+Enter), not at the end.
    pub top: bool,
}

impl NewCard {
    /// A new card with a title at the end (or the top) of a column.
    pub fn new(column: impl Into<String>, title: impl Into<String>, top: bool) -> Self {
        Self {
            column: column.into(),
            title: title.into(),
            top,
        }
    }
}

/// What a column's short menu can do.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnAction {
    /// Rename.
    Rename,
    /// Set the work-in-progress limit.
    SetLimit,
    /// Fold the column away.
    Collapse,
    /// Open the table of columns.
    EditColumns,
}

/// What a context menu is for.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextTarget {
    /// A card.
    Card(String),
    /// A column (its head).
    Column(String),
    /// The empty area of the board.
    Board,
}

/// The filter the bar shows; the program filters, the board only says what is asked.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterState {
    /// Free text.
    pub text: String,
    /// Chosen people (references the program understands).
    pub people: Vec<String>,
    /// Chosen labels.
    pub labels: Vec<String>,
}

impl FilterState {
    /// Nothing is filtered.
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.people.is_empty() && self.labels.is_empty()
    }
}
