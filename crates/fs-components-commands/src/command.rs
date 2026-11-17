//! One command: what a program can do, with its state, toggle, symbol and shortcuts.
//!
//! Pattern: Command (a command is an object with a name and a state; the program runs it, the model only describes it) as a
//! Value object; builders return the changed value, so a command is written in one expression.

use crate::shortcut::Shortcut;

/// The stable name of a command: a freedesktop action name (`edit-copy`) or `<program>.<command>`. Not empty.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(String);

impl CommandId {
    /// The id with this name (an empty one is reported by validation, not refused here, so every problem shows at once).
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CommandId {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// The key of a text in the MF2 files of the program (ADR 0057). The model never holds the words themselves.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextKey(String);

impl TextKey {
    /// The key with this name.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TextKey {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

/// Can the command be run now? "Never greyed out silently": everything but `Enabled` carries the key of the text that says why.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// It can be run.
    Enabled,
    /// It exists but cannot be run now (nothing selected, nothing to save, ...).
    Disabled {
        /// Why; shown with the line.
        reason: TextKey,
    },
    /// The program cannot do it at all in this setting (a feature the data does not have).
    Unsupported {
        /// Why; shown with the line.
        reason: TextKey,
    },
}

impl State {
    /// Is the command locked (`Disabled` or `Unsupported`)?
    #[must_use]
    pub fn is_locked(&self) -> bool {
        !matches!(self, Self::Enabled)
    }

    /// The key of the reason, for a locked command.
    #[must_use]
    pub fn reason(&self) -> Option<&TextKey> {
        match self {
            Self::Enabled => None,
            Self::Disabled { reason } | Self::Unsupported { reason } => Some(reason),
        }
    }
}

/// Is the command a mark in a menu?
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Toggle {
    /// An ordinary line.
    None,
    /// A line with a check mark of its own (a view option that is on or off).
    Check {
        /// Is it on?
        on: bool,
    },
    /// One of a group of lines of which exactly one is on (the view as list or tree).
    Radio {
        /// The name of the group; commands with the same name belong together.
        group: String,
        /// Is it the chosen one?
        on: bool,
    },
}

/// What a program publishes: one thing it can do.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// Stable name.
    pub id: CommandId,
    /// The key of the text of the line.
    pub label: TextKey,
    /// The name of the symbol (freedesktop icon naming: `edit-copy`).
    pub icon: Option<String>,
    /// The shortcuts the command starts with, the first is shown.
    pub shortcuts: Vec<Shortcut>,
    /// Can it be run now.
    pub state: State,
    /// Check mark or radio.
    pub toggle: Toggle,
}

impl Command {
    /// An enabled command without symbol, shortcut and mark.
    pub fn new(id: impl Into<CommandId>, label: impl Into<TextKey>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcuts: Vec::new(),
            state: State::Enabled,
            toggle: Toggle::None,
        }
    }

    /// With this symbol.
    #[must_use]
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    /// With one more shortcut.
    #[must_use]
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcuts.push(shortcut);
        self
    }

    /// Locked for now, for this reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<TextKey>) -> Self {
        self.state = State::Disabled {
            reason: reason.into(),
        };
        self
    }

    /// Not possible here, for this reason.
    #[must_use]
    pub fn unsupported(mut self, reason: impl Into<TextKey>) -> Self {
        self.state = State::Unsupported {
            reason: reason.into(),
        };
        self
    }

    /// A line with its own check mark.
    #[must_use]
    pub fn check(mut self, on: bool) -> Self {
        self.toggle = Toggle::Check { on };
        self
    }

    /// One line of the radio group `group`.
    #[must_use]
    pub fn radio(mut self, group: impl Into<String>, on: bool) -> Self {
        self.toggle = Toggle::Radio {
            group: group.into(),
            on,
        };
        self
    }
}
