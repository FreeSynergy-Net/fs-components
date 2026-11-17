//! What is wrong with a command list or an arrangement, as values.
//!
//! Pattern: Value object — validation collects every problem instead of stopping at the first, so a program author sees all at once.

use std::fmt;

use crate::command::CommandId;

/// One problem. `path` is the position in the arrangement tree (index per level).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// A command without a name.
    EmptyCommandId,
    /// Two commands with the same id.
    DuplicateCommand(CommandId),
    /// A locked command whose reason has no text key: "never greyed out silently".
    LockedWithoutReason(CommandId),
    /// A radio group where not exactly one line is on (`on` is how many are).
    RadioGroupOn {
        /// The group.
        group: String,
        /// How many lines of it are on.
        on: usize,
    },
    /// The arrangement names a command that does not exist.
    UnknownCommand {
        /// The id.
        id: CommandId,
        /// Where.
        path: Vec<usize>,
    },
    /// A command stands twice in one arrangement.
    CommandTwice(CommandId),
    /// A separator first or last in a level (groups count as part of the level around them).
    SeparatorAtEdge {
        /// Where.
        path: Vec<usize>,
    },
    /// Two separators in a row.
    SeparatorTwice {
        /// Where.
        path: Vec<usize>,
    },
    /// A submenu without a line that can be chosen.
    EmptySubmenu {
        /// Where.
        path: Vec<usize>,
    },
    /// A submenu or a group without a name.
    EmptyName {
        /// Where.
        path: Vec<usize>,
    },
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCommandId => write!(f, "a command has no id"),
            Self::DuplicateCommand(id) => write!(f, "two commands have the id {:?}", id.as_str()),
            Self::LockedWithoutReason(id) => {
                write!(f, "command {:?} is locked without a reason", id.as_str())
            }
            Self::RadioGroupOn { group, on } => {
                write!(f, "radio group {group:?} has {on} lines on, not one")
            }
            Self::UnknownCommand { id, path } => {
                write!(f, "unknown command {:?} at {path:?}", id.as_str())
            }
            Self::CommandTwice(id) => write!(f, "command {:?} stands twice", id.as_str()),
            Self::SeparatorAtEdge { path } => write!(f, "separator at the edge at {path:?}"),
            Self::SeparatorTwice { path } => write!(f, "two separators in a row at {path:?}"),
            Self::EmptySubmenu { path } => write!(f, "empty submenu at {path:?}"),
            Self::EmptyName { path } => write!(f, "no name at {path:?}"),
        }
    }
}
