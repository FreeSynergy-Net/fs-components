//! Where commands are offered: a tree of items per place.
//!
//! Pattern: Composite — a submenu and a group hold items again; the arrangement only names commands, it never holds their state.

use crate::command::{CommandId, TextKey};

/// One item of an arrangement.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuItem {
    /// A command, by id.
    Command(CommandId),
    /// A line that opens the next level.
    Submenu {
        /// The key of its text.
        label: TextKey,
        /// Its symbol.
        icon: Option<String>,
        /// The next level.
        children: Vec<MenuItem>,
    },
    /// A thin line between two parts.
    Separator,
    /// Items that belong together (and may get a look of their own, like a row of symbols); no line of its own.
    Group {
        /// Its name.
        id: String,
        /// Its items; they count as part of the level around the group (as one row of symbols if `inline_icons`).
        children: Vec<MenuItem>,
        /// Shown as one row of symbols (the quick actions on top of the Explorer menu): only commands with a symbol, no
        /// text per line. A view that has no such row (the title-bar menu) shows them as ordinary lines.
        inline_icons: bool,
    },
    /// A place where the frontend puts something that is no command (a search field); the name says which.
    Placeholder(String),
}

impl MenuItem {
    /// A command line.
    pub fn command(id: impl Into<CommandId>) -> Self {
        Self::Command(id.into())
    }

    /// A submenu without symbol.
    pub fn submenu(label: impl Into<TextKey>, children: Vec<MenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            icon: None,
            children,
        }
    }

    /// A group.
    pub fn group(id: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self::Group {
            id: id.into(),
            children,
            inline_icons: false,
        }
    }

    /// A group shown as a row of symbols (`inline_icons`): every child is a command with a symbol.
    pub fn icon_row(id: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self::Group {
            id: id.into(),
            children,
            inline_icons: true,
        }
    }
}

/// The tree of one place: `menubar`, `context:file`, `toolbar`, `palette`, ...
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrangement {
    /// The name of the place.
    pub place: String,
    /// The first level.
    pub items: Vec<MenuItem>,
}

impl Arrangement {
    /// The arrangement of `place`.
    pub fn new(place: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            place: place.into(),
            items,
        }
    }
}
