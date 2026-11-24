//! The commands of the board that a context menu or a shortcut can run: ids of the command model, run by the workbench.
//!
//! Pattern: Command (ids as plain text, so a menu line, a shortcut and a workflow block name the same thing). The shell opens
//! the menu (`fs-components-app`); the workbench says which lines a card has and runs the id it gets back.

/// Open the card.
pub const OPEN: &str = "board.open";
/// Move the card up one place.
pub const UP: &str = "board.up";
/// Move the card down one place.
pub const DOWN: &str = "board.down";
/// Move the card to the top of its column.
pub const TOP: &str = "board.top";
/// Move the card to the end of its column.
pub const BOTTOM: &str = "board.bottom";
/// Prefix of "move to column": `board.to:<column key>`.
pub const TO_PREFIX: &str = "board.to:";
/// Take the last move back.
pub const UNDO: &str = "board.undo";
/// Open the quick entry of a column.
pub const NEW: &str = "board.new";

/// The id that moves a card to `column`.
pub fn to_column(column: &str) -> String {
    format!("{TO_PREFIX}{column}")
}

/// The column of a "move to" id.
pub fn column_of(id: &str) -> Option<&str> {
    id.strip_prefix(TO_PREFIX)
}

/// One line of a card's menu: an id, and whether it can be run now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The command id.
    pub id: String,
    /// It can be run now (a card at the top has no "up").
    pub enabled: bool,
}

/// The lines of a card's menu for a card at `index` of `len` shown cards: open, up, down, top, end, then one "move to" per other
/// column (the shell puts those in the submenu "Move to").
pub fn card_lines(index: usize, len: usize, other_columns: &[String]) -> Vec<Line> {
    let line = |id: &str, enabled: bool| Line {
        id: id.to_owned(),
        enabled,
    };
    let mut lines = vec![
        line(OPEN, true),
        line(UP, index > 0),
        line(DOWN, index + 1 < len),
        line(TOP, index > 0),
        line(BOTTOM, index + 1 < len),
    ];
    lines.extend(other_columns.iter().map(|c| line(&to_column(c), true)));
    lines
}
