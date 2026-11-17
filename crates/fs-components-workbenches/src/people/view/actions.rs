//! What the person can do, as callbacks.
//!
//! Pattern: none of the table fits (plain closures, no listener trait) — the widget decides what each one does: change the
//! state and draw again.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use fs_components_commands::Trigger;
use fs_components_widgets::keys::KeyEvent;

/// A plain action.
pub(in crate::people) type Act = Arc<dyn Fn() + Send + Sync>;
/// An action about something named by a key.
pub(in crate::people) type ActOn = Arc<dyn Fn(&str) + Send + Sync>;
/// The person typed text into the field with this key.
pub(in crate::people) type Typed = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Run a command by id; the second is what a card hands (the text to copy, the address to write to).
pub(in crate::people) type RunCmd = Arc<dyn Fn(&str, Option<&str>) + Send + Sync>;
/// A menu was asked for: where, and for what.
pub(in crate::people) type OpenMenu = Arc<dyn Fn(MenuFor, Trigger) + Send + Sync>;

/// A key the keyboard of a row did not use itself; the first is the key of the row, if it was a row.
pub(in crate::people) type KeyOn = Arc<dyn Fn(Option<&str>, &KeyEvent) + Send + Sync>;

/// What a context menu is for.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuFor {
    /// A row of the list (a press on it first makes it the selection, as the model says).
    Row(String),
    /// Empty space of the list.
    Empty,
    /// The menu of the list (the three lines in its bar).
    List,
    /// The menu of the person shown (the three dots).
    Person,
}

/// The things the view reports.
#[derive(Clone)]
pub(in crate::people) struct PeopleActions {
    /// A row was chosen.
    pub select: ActOn,
    /// Show more rows.
    pub more: Act,
    /// Save.
    pub save: Act,
    /// Cancel the edit.
    pub cancel: Act,
    /// Go on editing after being asked.
    pub keep_editing: Act,
    /// Really discard the changes.
    pub confirm_discard: Act,
    /// Really delete.
    pub confirm_delete: Act,
    /// Keep the person after being asked.
    pub keep: Act,
    /// Add a field to this group.
    pub add_field: ActOn,
    /// Text typed.
    pub typed: Typed,
    /// A row took a right click: the list around the rows leaves it alone (a press reaches every ancestor).
    pub claimed: Arc<AtomicBool>,
    /// A key that may be a shortcut.
    pub key: KeyOn,
    /// A command, by id.
    pub run: RunCmd,
    /// A context menu was asked for.
    pub menu: OpenMenu,
}
