//! The plain data of the board workbench: what the program asks, what it answers, what the reader asked for.
//!
//! Pattern: Value objects. No program type appears here: a column key and a card key are opaque texts (a status, a stage,
//! a task id are the program's), and every sentence is already in the reader's language (the program's data).

/// Why something is not possible, said by the program: a code for logs and tests, a sentence for the reader.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reason {
    /// A short code (`wip-locked`, `claimed`, `no-right`), never shown alone.
    pub code: String,
    /// The sentence the reader sees.
    pub text: String,
}

impl Reason {
    /// A reason with a code and the sentence.
    pub fn new(code: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            text: text.into(),
        }
    }
}

/// A question the program asks before it does a move (an agent holds the card: move it anyway?).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    /// The question.
    pub text: String,
}

impl Question {
    /// A question.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// What the program says to a request of the board.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Done; the next `columns()` shows it.
    Accepted,
    /// Not done; the card goes back and the reason is said.
    Refused(Reason),
    /// Ask first; on yes the same request comes again with `confirmed` set.
    Ask(Question),
}

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

#[allow(clippy::struct_excessive_bools)] // four independent yes/no capabilities
/// What the board supports now; a control of an unsupported thing is left out, never shown dead without a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Supports {
    /// Cards can be moved.
    pub reorder: bool,
    /// The foot of a column takes a new card.
    pub quick_add: bool,
    /// The table of columns can be opened.
    pub columns_edit: bool,
    /// A context menu is offered.
    pub context: bool,
}

impl Default for Supports {
    fn default() -> Self {
        Self {
            reorder: true,
            quick_add: true,
            columns_edit: true,
            context: true,
        }
    }
}
