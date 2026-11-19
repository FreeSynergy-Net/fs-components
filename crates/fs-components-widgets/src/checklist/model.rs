//! The pure state of a checklist.
//!
//! Pattern: State — the items, the cursor, what a key does.

use crate::keys::{Key, KeyEvent};

/// One item.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// What the caller gets back; part of the ids.
    pub key: String,
    /// What the person reads.
    pub text: String,
    /// Ticked off.
    pub done: bool,
}

impl Item {
    /// An item that is not done.
    pub fn new(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            text: text.into(),
            done: false,
        }
    }

    /// The same item, done.
    #[must_use]
    pub fn done(mut self) -> Self {
        self.done = true;
        self
    }
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the list.
    Ignored,
    /// The cursor moved.
    Moved,
    /// The item with this key was ticked (`true`) or unticked.
    Toggled(String, bool),
    /// The item with this key was removed.
    Removed(String),
    /// The item with this key moved to this place.
    Reordered(String, usize),
}

/// The items and the cursor.
#[derive(Clone, Debug, Default)]
pub struct State {
    items: Vec<Item>,
    cursor: usize,
    next: usize,
}

impl State {
    /// A list of these items, the cursor on the first.
    pub fn new(items: Vec<Item>) -> Self {
        let next = items.len();
        Self {
            items,
            cursor: 0,
            next,
        }
    }

    /// The items.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The index of the item under the cursor.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Add an item at the end (a blank text is no item); its key is `n<number>`, a number that was not used before. `None` for a blank.
    pub fn add(&mut self, text: &str) -> Option<String> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let key = loop {
            let k = format!("n{}", self.next);
            self.next += 1;
            if !self.items.iter().any(|i| i.key == k) {
                break k;
            }
        };
        self.items.push(Item::new(key.clone(), text));
        Some(key)
    }

    /// Put the cursor on the item with this key.
    pub fn go(&mut self, key: &str) {
        if let Some(at) = self.items.iter().position(|i| i.key == key) {
            self.cursor = at;
        }
    }

    /// Tick or untick the item with this key; the new state, `None` when there is no such item.
    pub fn toggle(&mut self, key: &str) -> Option<bool> {
        let item = self.items.iter_mut().find(|i| i.key == key)?;
        item.done = !item.done;
        Some(item.done)
    }

    /// Remove the item with this key; the cursor keeps its place, or goes to the last item.
    pub fn remove(&mut self, key: &str) -> bool {
        let Some(at) = self.items.iter().position(|i| i.key == key) else {
            return false;
        };
        self.items.remove(at);
        if at < self.cursor {
            self.cursor -= 1;
        }
        self.cursor = self.cursor.min(self.items.len().saturating_sub(1));
        true
    }

    /// What a key does.
    pub fn press(&mut self, event: KeyEvent) -> Outcome {
        let last = self.items.len().saturating_sub(1);
        let Some(here) = self.items.get(self.cursor).map(|i| i.key.clone()) else {
            return Outcome::Ignored;
        };
        match event.key {
            Key::Up if event.alt && self.cursor > 0 => self.shift(self.cursor - 1, &here),
            Key::Down if event.alt && self.cursor < last => self.shift(self.cursor + 1, &here),
            Key::Down if !event.alt && self.cursor < last => self.step(self.cursor + 1),
            Key::Up if !event.alt && self.cursor > 0 => self.step(self.cursor - 1),
            Key::Home if self.cursor > 0 => self.step(0),
            Key::End if self.cursor < last => self.step(last),
            Key::Space | Key::Enter | Key::Activate => {
                let done = self.toggle(&here).unwrap_or(false);
                Outcome::Toggled(here, done)
            }
            Key::Delete | Key::Backspace => {
                self.remove(&here);
                Outcome::Removed(here)
            }
            _ => Outcome::Ignored,
        }
    }

    fn step(&mut self, to: usize) -> Outcome {
        self.cursor = to;
        Outcome::Moved
    }

    fn shift(&mut self, to: usize, key: &str) -> Outcome {
        self.items.swap(self.cursor, to);
        self.cursor = to;
        Outcome::Reordered(key.to_owned(), to)
    }
}
