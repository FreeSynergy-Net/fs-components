//! The pure state of a chip row.
//!
//! Pattern: State — the chips, the cursor, what a key does.

use crate::keys::Key;

/// One chip.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    /// What the caller gets back; part of the id.
    pub key: String,
    /// What the person reads.
    pub label: String,
}

impl Chip {
    /// A chip.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the row.
    Ignored,
    /// The cursor moved.
    Moved,
    /// The chip with this key was removed.
    Removed(String),
}

/// The chips and the cursor.
#[derive(Clone, Debug, Default)]
pub struct State {
    chips: Vec<Chip>,
    cursor: usize,
}

impl State {
    /// A row of these chips, the cursor on the first.
    pub fn new(chips: Vec<Chip>) -> Self {
        Self { chips, cursor: 0 }
    }

    /// The chips.
    pub fn chips(&self) -> &[Chip] {
        &self.chips
    }

    /// The index of the chip under the cursor.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Add a chip at the end; false when the key is there already.
    pub fn add(&mut self, chip: Chip) -> bool {
        if self.chips.iter().any(|c| c.key == chip.key) {
            return false;
        }
        self.chips.push(chip);
        true
    }

    /// Remove the chip with this key; the cursor keeps its place, or goes to the last chip.
    pub fn remove(&mut self, key: &str) -> bool {
        let Some(at) = self.chips.iter().position(|c| c.key == key) else {
            return false;
        };
        self.chips.remove(at);
        if at < self.cursor {
            self.cursor -= 1;
        }
        self.cursor = self.cursor.min(self.chips.len().saturating_sub(1));
        true
    }

    /// Put the cursor on the chip with this key.
    pub fn go(&mut self, key: &str) {
        if let Some(at) = self.chips.iter().position(|c| c.key == key) {
            self.cursor = at;
        }
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        let last = self.chips.len().saturating_sub(1);
        let to = match key {
            Key::Right if self.cursor < last => self.cursor + 1,
            Key::Left if self.cursor > 0 => self.cursor - 1,
            Key::Home if self.cursor > 0 => 0,
            Key::End if self.cursor < last => last,
            Key::Delete | Key::Backspace => {
                let Some(chip) = self.chips.get(self.cursor) else {
                    return Outcome::Ignored;
                };
                let key = chip.key.clone();
                self.remove(&key);
                return Outcome::Removed(key);
            }
            _ => return Outcome::Ignored,
        };
        self.cursor = to;
        Outcome::Moved
    }
}
