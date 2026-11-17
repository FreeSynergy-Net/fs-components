//! Keys to commands (optional).
//!
//! Pattern: Value object, as `share-dialog/bindings`: plain data. A key typed into the search field never reaches these (the
//! key guard ignores keys while a text input has the keyboard).

use crate::keys::{Key, KeyEvent};

/// One step of the picker.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Choose or unchoose the row under the cursor.
    Toggle,
    /// "Assign to me".
    Me,
    /// Confirm.
    Confirm,
    /// Cancel.
    Cancel,
}

/// Which key does what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    map: Vec<(Key, Command)>,
}

impl Default for Bindings {
    /// Up/Down move, Space or Enter (arriving as `Activate`) choose or unchoose, Escape cancels. Confirm has no key by
    /// default (a window keeps Enter for its own buttons: Tab to Done); a frontend may bind `Command::Confirm` or
    /// `Command::Me` to a key of its own.
    fn default() -> Self {
        Self {
            map: vec![
                (Key::Up, Command::Up),
                (Key::Down, Command::Down),
                (Key::Activate, Command::Toggle),
                (Key::Space, Command::Toggle),
                (Key::Escape, Command::Cancel),
            ],
        }
    }
}

impl Bindings {
    /// No keys; add with [`Bindings::bind`].
    pub fn none() -> Self {
        Self { map: Vec::new() }
    }

    /// `key` does `command` (replacing what it did).
    #[must_use]
    pub fn bind(mut self, key: Key, command: Command) -> Self {
        self.map.retain(|(k, _)| *k != key);
        self.map.push((key, command));
        self
    }

    /// The command for a press (none with Ctrl or Alt held, or for an unbound key).
    pub fn command(&self, event: &KeyEvent) -> Option<Command> {
        if event.ctrl || event.alt {
            return None;
        }
        self.map
            .iter()
            .find(|(k, _)| *k == event.key)
            .map(|(_, c)| *c)
    }
}
