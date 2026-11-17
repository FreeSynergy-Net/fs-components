//! What a program says about its menus: labels and entries, nothing about how they look.
//!
//! Pattern: Value object — `MenuEntry` and `Menu` are values, compared by value; an entry is named by a stable id
//! and one handler of the program receives that id. There is no operation enum, so this is not a Command.

use std::sync::Arc;

/// The one function of a program that receives the id of a chosen menu entry.
pub type MenuHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// What kind of mark a line carries when `checked` says it is on.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mark {
    /// The classic mark of this shell: a tick, drawn for a chosen line of a group (`checked`). Programs written before the
    /// model use this one and keep their look.
    #[default]
    Plain,
    /// A line with a check mark of its own (an option that is on or off): a tick, role `menuitemcheckbox`.
    Check,
    /// One of a group where exactly one is on: a dot, role `menuitemradio`.
    Radio,
}

/// One line of a menu.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuEntry {
    /// Stable name, unique among all entries of the program; the handler receives it.
    pub id: String,
    /// What the menu shows; comes from the program.
    pub label: String,
    /// Shown, but cannot be chosen.
    pub disabled: bool,
    /// Marked as the chosen one of a group of lines (the shell draws the mark; the label stays the plain name).
    pub checked: bool,
    /// The next level: a line with children opens them instead of choosing itself.
    pub children: Vec<MenuEntry>,
    /// A thin line between two parts; all other fields mean nothing then. It is no choice and has no id of its own.
    pub separator: bool,
    /// How `checked` is drawn.
    pub mark: Mark,
    /// The name of the symbol at the left of the label (freedesktop icon naming).
    pub icon: Option<String>,
    /// The shortcut as the line shows it at the right (`Strg+S`, plattformgerecht); it is shown, never run (M2).
    pub shortcut: Option<String>,
    /// The shortcut as stored (`aria-keyshortcuts`), for assistive technology.
    pub shortcut_aria: Option<String>,
    /// Why a disabled line is disabled; shown under the label.
    pub reason: Option<String>,
}

impl MenuEntry {
    /// An entry that can be chosen.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            checked: false,
            children: Vec::new(),
            separator: false,
            mark: Mark::Plain,
            icon: None,
            shortcut: None,
            shortcut_aria: None,
            reason: None,
        }
    }

    /// A thin line between two parts of a menu.
    pub fn separator() -> Self {
        Self {
            separator: true,
            ..Self::new("", "")
        }
    }

    /// A line that opens a next level of lines. It is no choice itself: its `id` is only a name for the element, and
    /// the handler never receives it.
    pub fn submenu(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<MenuEntry>,
    ) -> Self {
        Self {
            children,
            ..Self::new(id, label)
        }
    }

    /// The same entry, marked (or not) as the chosen one.
    #[must_use]
    pub fn checked(mut self, on: bool) -> Self {
        self.checked = on;
        self
    }

    /// A line with a check mark of its own, on or off.
    #[must_use]
    pub fn check(mut self, on: bool) -> Self {
        self.mark = Mark::Check;
        self.checked = on;
        self
    }

    /// A line of a group of which one is on.
    #[must_use]
    pub fn radio(mut self, on: bool) -> Self {
        self.mark = Mark::Radio;
        self.checked = on;
        self
    }

    /// With a symbol (freedesktop icon name).
    #[must_use]
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    /// With a shortcut shown at the right: the text to show and the stored form.
    #[must_use]
    pub fn shortcut(mut self, shown: impl Into<String>, aria: impl Into<String>) -> Self {
        self.shortcut = Some(shown.into());
        self.shortcut_aria = Some(aria.into());
        self
    }

    /// Disabled, and says why (the reason is shown under the label).
    #[must_use]
    pub fn disabled_because(mut self, reason: impl Into<String>) -> Self {
        self.disabled = true;
        self.reason = Some(reason.into());
        self
    }

    /// Does this line carry a mark (tick or dot) of any kind, on or off?
    pub(crate) fn has_mark(&self) -> bool {
        self.checked || self.mark != Mark::Plain
    }

    /// This entry and all below it, depth first.
    pub(crate) fn walk<'a>(&'a self, out: &mut Vec<&'a MenuEntry>) {
        out.push(self);
        for child in &self.children {
            child.walk(out);
        }
    }

    /// The same entry, shown but not to be chosen.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// One menu of the title bar. A menu without entries is not shown.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    /// What the title bar shows; comes from the program.
    pub label: String,
    /// The lines of the menu, in this order.
    pub entries: Vec<MenuEntry>,
}

impl Menu {
    /// A menu with this label and these entries.
    pub fn new(label: impl Into<String>, entries: Vec<MenuEntry>) -> Self {
        Self {
            label: label.into(),
            entries,
        }
    }
}

impl Menu {
    /// All entries of this menu, also those of the next levels, depth first.
    pub(crate) fn all_entries(&self) -> Vec<&MenuEntry> {
        let mut out = Vec::new();
        for entry in &self.entries {
            entry.walk(&mut out);
        }
        out
    }
}
