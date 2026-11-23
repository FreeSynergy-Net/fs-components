//! The standard menu of every program as fixed commands `app.*` (ADR 0065: Program, View, Help; Ctrl+Q, F1).
//!
//! Pattern: Registry (fixed commands) + Template method (the arrangement of the menu bar). The commands live in the model like every
//! other command, so the palette, a context menu and a block of a workflow reach them too, and the shell answers them in every
//! frontend (a program only adds its own). The ids are fixed and reserved: a program command may not use the prefix `app.`
//! ([`crate::Problem::ReservedId`]). The words are the keys of the shell's own texts (`shell-*`, German and English).

use crate::arrangement::MenuItem;
use crate::command::Command;
use crate::commands::Commands;
use crate::parameter::{Param, ParamType};
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

/// The fixed commands of the shell and then the program's. The three of the notification module ([`notification_commands`]) are
/// not merged here (the menu of the shell stays as it is); their ids are reserved all the same.
///
/// # Errors
/// [`Problem::ReservedId`] for every program command whose id starts with `app.` or `notification.`; nothing is merged then.
pub fn with_standard(program: &Commands) -> Result<Commands, Vec<Problem>> {
    let problems: Vec<Problem> = program
        .iter()
        .filter(|c| {
            c.id.as_str().starts_with(PREFIX) || c.id.as_str().starts_with(NOTIFICATION_PREFIX)
        })
        .map(|c| Problem::ReservedId(c.id.clone()))
        .collect();
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(program
        .iter()
        .fold(standard_commands(), |all, c| all.with(c.clone())))
}

/// The prefix of the commands of the notification module (ADR 0106 point 3); reserved like `app.`.
pub const NOTIFICATION_PREFIX: &str = "notification.";
/// Tell something (a message or a failure): returns the id of the notification.
pub const NOTIFICATION_TELL: &str = "notification.tell";
/// Ask something (a question with buttons): returns the id; the answer arrives as an event of the module.
pub const NOTIFICATION_ASK: &str = "notification.ask";
/// Close a notification by its id.
pub const NOTIFICATION_CLOSE: &str = "notification.close";

/// The three fixed commands of the notification module (ADR 0106 point 3), described as every command is: typed parameters and a
/// result, so a workflow block, the palette, MCP and the command line reach them. They are carried out by `fs-notification`
/// (`Notifications::run`). The texts and titles are keys of MF2 texts (`title` and `body` name a text, they are never a sentence);
/// `actions` are entries `id=text-key`, `params` are entries `name=value` for the placeholders of the texts.
#[must_use]
pub fn notification_commands() -> Commands {
    let text = |name: &str, label: &str| Param::new(name, label).of(ParamType::Text);
    let texts =
        |name: &str, label: &str| Param::new(name, label).of(ParamType::list(ParamType::Text));
    let message = |command: Command| {
        command
            .param(text("title", "shell-notification-title"))
            .param(text("body", "shell-notification-body").optional())
            .param(
                Param::new("urgency", "shell-notification-urgency")
                    .of(ParamType::choice(["low", "normal", "critical"]))
                    .optional(),
            )
            .param(text("category", "shell-notification-category").optional())
            .param(text("scope", "shell-notification-scope").optional())
            .param(text("replaces", "shell-notification-replaces").optional())
            .param(
                Param::new("expire-ms", "shell-notification-expire")
                    .of(ParamType::Number)
                    .optional(),
            )
            .param(texts("params", "shell-notification-params").optional())
    };
    Commands::new()
        .with(
            message(
                Command::new(NOTIFICATION_TELL, "shell-notification-tell")
                    .icon("dialog-information"),
            )
            .param(texts("actions", "shell-notification-actions").optional())
            .returns(ParamType::Text),
        )
        .with(
            message(
                Command::new(NOTIFICATION_ASK, "shell-notification-ask").icon("dialog-question"),
            )
            .param(texts("actions", "shell-notification-actions"))
            .returns(ParamType::Text),
        )
        .with(
            Command::new(NOTIFICATION_CLOSE, "shell-notification-close")
                .icon("window-close")
                .param(text("id", "shell-notification-id")),
        )
}
