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
    /// A parameter without a name, or a name used twice in one command.
    ParamName {
        /// The command.
        command: CommandId,
        /// The name (empty if there is none).
        name: String,
    },
    /// A parameter without a type: a block of a workflow could not ask for it.
    ParamWithoutType {
        /// The command.
        command: CommandId,
        /// The parameter.
        name: String,
    },
    /// A type that cannot be filled: a choice without options, an object reference without a kind.
    ParamHollow {
        /// The command.
        command: CommandId,
        /// The parameter (`result` for the result).
        name: String,
    },
    /// A program command with an id of the shell (`app.`): those are fixed.
    ReservedId(CommandId),
    /// A row of symbols with something in it that is no command.
    IconRowNotCommand {
        /// Where.
        path: Vec<usize>,
    },
    /// A command in a row of symbols has no symbol.
    IconRowWithoutIcon(CommandId),
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
            Self::ParamName { command, name } => write!(
                f,
                "command {:?} has a parameter with no name or the name {name:?} twice",
                command.as_str()
            ),
            Self::ParamWithoutType { command, name } => write!(
                f,
                "parameter {name:?} of command {:?} has no type",
                command.as_str()
            ),
            Self::ParamHollow { command, name } => write!(
                f,
                "parameter {name:?} of command {:?} has a type that cannot be filled",
                command.as_str()
            ),
            Self::ReservedId(id) => write!(f, "{:?} is an id of the shell (app.*)", id.as_str()),
            Self::IconRowNotCommand { path } => {
                write!(f, "a row of symbols holds something but commands at {path:?}")
            }
            Self::IconRowWithoutIcon(id) => {
                write!(f, "command {:?} in a row of symbols has no symbol", id.as_str())
            }
        }
    }
}
