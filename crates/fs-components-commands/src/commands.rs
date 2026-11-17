//! The list of the commands of a program.
//!
//! Pattern: Registry — the one place that knows every command by its id; arrangements only name them.

use crate::command::{Command, CommandId};

/// All commands a program publishes, in the order it gave them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Commands {
    pub(crate) list: Vec<Command>,
}

impl Commands {
    /// No commands.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// With one more command. A second command with the same id is kept; [`crate::validate_commands`] reports it.
    #[must_use]
    pub fn with(mut self, command: Command) -> Self {
        self.list.push(command);
        self
    }

    /// The command with this id (the first, if there are two).
    #[must_use]
    pub fn get(&self, id: &CommandId) -> Option<&Command> {
        self.list.iter().find(|c| &c.id == id)
    }

    /// All commands in order.
    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.list.iter()
    }

    /// How many there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Are there none?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}
