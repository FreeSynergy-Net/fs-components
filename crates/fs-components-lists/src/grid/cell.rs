//! What a cell shows: text by default, any element when the caller says so.
//!
//! Pattern: Strategy – the caller gives a [`CellView`] that answers, per cell and per head cell, "this element instead of the
//! text"; [`TextCells`] (the default) answers nothing, so every grid built before keeps showing text and needs no change.

use blinc_layout::div::Div;

/// One cell as the model knows it (a value, no element).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    /// A cell in its own column; the words are the text shown when no [`CellView`] gives an element.
    Text(String),
    /// A cell that spans ALL columns of its row (a parent row's head, a lane head): the row has this one cell
    /// (`aria-colspan` = the number of columns). The words are its label.
    Spans(String),
}

impl Cell {
    /// The words of the cell.
    pub fn words(&self) -> &str {
        match self {
            Self::Text(w) | Self::Spans(w) => w,
        }
    }

    /// Whether the cell spans every column.
    pub fn spans(&self) -> bool {
        matches!(self, Self::Spans(_))
    }
}

/// Where a cell is and how it stands; what a [`CellView`] is asked about.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellAt<'a> {
    /// The key of the row.
    pub row: &'a str,
    /// The key of the column (for a spanning cell: the first column).
    pub col: &'a str,
    /// The row is an open parent (treegrid); false for a plain row.
    pub open: bool,
    /// The keys are inside this cell (variant `cells-hold`, after Enter): the content has them, not the grid.
    pub inside: bool,
}

impl<'a> CellAt<'a> {
    /// A place.
    pub fn new(row: &'a str, col: &'a str, open: bool, inside: bool) -> Self {
        Self {
            row,
            col,
            open,
            inside,
        }
    }
}

/// What a grid shows in its cells. Every method may answer `None`: then the grid shows the text it holds (the head: the label).
pub trait CellView: Send + Sync {
    /// The element of a body cell (also a spanning cell: `at.col` is then the first column).
    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        let _ = at;
        None
    }

    /// The element of the head cell of the column `col`.
    fn head(&self, col: &str) -> Option<Div> {
        let _ = col;
        None
    }
}

/// The default strategy: text everywhere.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextCells;

impl CellView for TextCells {}
