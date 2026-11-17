//! Arrangement and commands put together: the tree a view draws.
//!
//! Pattern: Interpreter-like resolution (arrangement x command list to a tree of values). One resolution for every view (menu
//! bar, context menu, palette), so they cannot disagree about a command.

use crate::arrangement::{Arrangement, MenuItem};
use crate::command::{Command, TextKey};
use crate::commands::Commands;
use crate::problem::Problem;
use crate::validate::{validate_arrangement, validate_commands};

/// One resolved item; groups are gone (their items stand in the level around them).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    /// A command with its state, toggle and shortcuts.
    Command(Command),
    /// A line that opens a next level.
    Submenu {
        /// Key of its text.
        label: TextKey,
        /// Its symbol.
        icon: Option<String>,
        /// The next level.
        children: Vec<Resolved>,
    },
    /// A thin line.
    Separator,
    /// A place for something that is no command.
    Placeholder(String),
    /// A row of symbols (a group with `inline_icons`): commands only, each with a symbol.
    IconRow(Vec<Command>),
}

/// Resolve `arrangement` against `commands`.
///
/// # Errors
/// All problems of both ([`validate_commands`], [`validate_arrangement`]); nothing is resolved then.
pub fn resolve(
    arrangement: &Arrangement,
    commands: &Commands,
) -> Result<Vec<Resolved>, Vec<Problem>> {
    let mut problems = validate_commands(commands);
    problems.extend(validate_arrangement(arrangement, commands));
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut out = Vec::new();
    walk(&arrangement.items, commands, &mut out);
    Ok(out)
}

fn walk(items: &[MenuItem], commands: &Commands, out: &mut Vec<Resolved>) {
    for item in items {
        match item {
            MenuItem::Command(id) => {
                if let Some(command) = commands.get(id) {
                    out.push(Resolved::Command(command.clone()));
                }
            }
            MenuItem::Submenu {
                label,
                icon,
                children,
            } => {
                let mut inner = Vec::new();
                walk(children, commands, &mut inner);
                out.push(Resolved::Submenu {
                    label: label.clone(),
                    icon: icon.clone(),
                    children: inner,
                });
            }
            MenuItem::Separator => out.push(Resolved::Separator),
            MenuItem::Group {
                children,
                inline_icons: true,
                ..
            } => {
                let row = children
                    .iter()
                    .filter_map(|c| match c {
                        MenuItem::Command(id) => commands.get(id).cloned(),
                        _ => None,
                    })
                    .collect();
                out.push(Resolved::IconRow(row));
            }
            MenuItem::Group { children, .. } => walk(children, commands, out),
            MenuItem::Placeholder(name) => out.push(Resolved::Placeholder(name.clone())),
        }
    }
}
