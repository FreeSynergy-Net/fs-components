//! The standard menu of every program as fixed commands `app.*` (ADR 0065: Program, View, Help; Ctrl+Q, F1).
//!
//! Pattern: Registry (fixed commands) + Template method (the arrangement of the menu bar). The commands live in the model like every
//! other command, so the palette, a context menu and a block of a workflow reach them too, and the shell answers them in every
//! frontend (a program only adds its own). The ids are fixed and reserved: a program command may not use the prefix `app.`
//! ([`crate::Problem::ReservedId`]). The words are the keys of the shell's own texts (`shell-*`, German and English).

use crate::arrangement::MenuItem;
use crate::command::Command;
use crate::commands::Commands;
use crate::problem::Problem;
use crate::shortcut::Shortcut;

/// The prefix of the commands of the shell.
pub const PREFIX: &str = "app.";
/// Close the window (Primary+Q).
pub const QUIT: &str = "app.quit";
/// Open the settings of the program (the program answers).
pub const SETTINGS: &str = "app.settings";
/// Open the help (F1; the program answers).
pub const HELP: &str = "app.help";
/// Show the about window (the program answers).
pub const ABOUT: &str = "app.about";

/// The fixed commands, in the order of the menu.
///
/// # Panics
/// Never: the two shortcuts are constants that parse.
#[must_use]
pub fn standard_commands() -> Commands {
    let key = |text: &str| Shortcut::parse(text).expect("a constant shortcut parses");
    Commands::new()
        .with(Command::new(SETTINGS, "shell-settings").icon("preferences-system"))
        .with(
            Command::new(QUIT, "shell-quit")
                .icon("application-exit")
                .shortcut(key("Primary+Q")),
        )
        .with(
            Command::new(HELP, "shell-help")
                .icon("help-browser")
                .shortcut(key("F1")),
        )
        .with(Command::new(ABOUT, "shell-about").icon("help-about"))
}

/// The menu bar: Program (Settings, Quit), View (only if the program brings lines of its own: the shell has nothing fixed for
/// it), Help (Help, About).
#[must_use]
pub fn standard_menubar(views: Vec<MenuItem>) -> Vec<MenuItem> {
    let mut bar = vec![MenuItem::submenu(
        "shell-menu-program",
        vec![
            MenuItem::command(SETTINGS),
            MenuItem::Separator,
            MenuItem::command(QUIT),
        ],
    )];
    if !views.is_empty() {
        bar.push(MenuItem::submenu("shell-menu-view", views));
    }
    bar.push(MenuItem::submenu(
        "shell-menu-help",
        vec![MenuItem::command(HELP), MenuItem::command(ABOUT)],
    ));
    bar
}

/// The fixed commands and then the program's.
///
/// # Errors
/// [`Problem::ReservedId`] for every program command whose id starts with `app.`; nothing is merged then.
pub fn with_standard(program: Commands) -> Result<Commands, Vec<Problem>> {
    let problems: Vec<Problem> = program
        .iter()
        .filter(|c| c.id.as_str().starts_with(PREFIX))
        .map(|c| Problem::ReservedId(c.id.clone()))
        .collect();
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(program
        .iter()
        .fold(standard_commands(), |all, c| all.with(c.clone())))
}
