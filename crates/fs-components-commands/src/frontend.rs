//! The command `layout.frontend` and the lines of the frontends (View menu, radio group), as data.
//!
//! Pattern: Registry (fixed command) + Value object (a row). Nothing is drawn here (the scaffold never draws): the rows say which
//! frontends a person can switch to, the radio lines are ordinary commands of the model, so the menu, the palette and a block of a
//! workflow reach them like every other command. The rows come from the catalog (`available_in`/`list`); this module only turns
//! them into commands. A row's `id` is also the key of its text (like the options of a `ParamType::Choice`), so a frontend's name
//! is a text in the files of its provider, never a word in code.

use crate::arrangement::MenuItem;
use crate::command::{Command, CommandId};
use crate::commands::Commands;
use crate::parameter::{Param, ParamType};

/// The prefix of the layout commands of the shell; reserved like `app.`.
pub const LAYOUT_PREFIX: &str = "layout.";
/// Switch the frontend of the program (parameter [`FRONTEND_PARAM`], one of the rows).
pub const FRONTEND: &str = "layout.frontend";
/// The name of the one parameter of [`FRONTEND`].
pub const FRONTEND_PARAM: &str = "frontend";
/// The radio group of the lines in the View menu.
pub const FRONTEND_GROUP: &str = "layout.frontend";

/// One frontend a person can switch to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendRow {
    /// Stable name of the frontend and the key of its text.
    pub id: String,
}

impl FrontendRow {
    /// The row of the frontend with this id.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// The id of the radio line of a frontend: `layout.frontend.<id>`.
#[must_use]
pub fn frontend_line_id(row: &FrontendRow) -> CommandId {
    CommandId::new(format!("{FRONTEND}.{}", row.id))
}

/// The command `layout.frontend`: one required parameter `frontend`, a choice of the rows (their ids are the text keys).
#[must_use]
pub fn frontend_command(rows: &[FrontendRow]) -> Command {
    Command::new(FRONTEND, "shell-frontend")
        .icon("view-split-left-right")
        .param(
            Param::new(FRONTEND_PARAM, "shell-frontend-param")
                .of(ParamType::choice(rows.iter().map(|r| r.id.clone()))),
        )
}

/// The radio lines, one per row, in the order of the rows; the row `chosen` is on. A `chosen` that is no row (or none) puts the
/// first row on, so the group is always valid. No rows, no lines.
#[must_use]
pub fn frontend_lines(rows: &[FrontendRow], chosen: Option<&str>) -> Commands {
    let on = rows
        .iter()
        .position(|r| Some(r.id.as_str()) == chosen)
        .unwrap_or(0);
    rows.iter()
        .enumerate()
        .fold(Commands::new(), |all, (i, row)| {
            all.with(
                Command::new(frontend_line_id(row), row.id.as_str()).radio(FRONTEND_GROUP, i == on),
            )
        })
}

/// The lines for the View menu (to give to [`crate::standard_menubar`]), in the order of the rows.
#[must_use]
pub fn frontend_menu(rows: &[FrontendRow]) -> Vec<MenuItem> {
    rows.iter()
        .map(|r| MenuItem::command(frontend_line_id(r)))
        .collect()
}
