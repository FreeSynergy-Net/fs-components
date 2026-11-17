//! Two commands with the same shortcut: shown, never decided silently (plan, section 1).
//!
//! Pattern: none — one pure function over the command list.

use crate::command::CommandId;
use crate::commands::Commands;
use crate::display::Platform;
use crate::shortcut::Shortcut;

/// One shortcut that more than one command has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    /// The shortcut as the first command spells it.
    pub shortcut: Shortcut,
    /// The commands that have it, in order, each once.
    pub commands: Vec<CommandId>,
}

/// All conflicts between the commands on `platform` (`Primary+S` and `Control+S` are the same key on Linux).
/// A command that lists the same shortcut twice does not conflict with itself.
#[must_use]
pub fn conflicts(commands: &Commands, platform: Platform) -> Vec<Conflict> {
    let mut found: Vec<Conflict> = Vec::new();
    for command in commands.iter() {
        for shortcut in &command.shortcuts {
            match found
                .iter_mut()
                .find(|c| c.shortcut.same_as(shortcut, platform))
            {
                Some(entry) => {
                    if !entry.commands.contains(&command.id) {
                        entry.commands.push(command.id.clone());
                    }
                }
                None => found.push(Conflict {
                    shortcut: shortcut.clone(),
                    commands: vec![command.id.clone()],
                }),
            }
        }
    }
    found.retain(|c| c.commands.len() > 1);
    found
}
