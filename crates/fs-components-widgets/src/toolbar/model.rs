//! The pure state of a toolbar.
//!
//! Pattern: State — the items, where the cursor is, which toggles are pressed, what a key does; no element, no callback.

use crate::keys::Key;
use crate::roving::{Axis, next};

/// What an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Does something when pressed.
    Button,
    /// Is on or off (`aria-pressed`).
    Toggle,
    /// A line between two groups; the cursor never stops on it.
    Separator,
}

/// One item of the bar.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// What the caller gets back; part of the id (`<id>-item-<key>`).
    pub key: String,
    /// What the person reads (and a screen reader says).
    pub label: String,
    /// What it is.
    pub kind: Kind,
    /// A toggle that is on.
    pub pressed: bool,
    /// Cannot be pressed; the keys skip it.
    pub disabled: bool,
}

impl Item {
    /// A button.
    pub fn button(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind: Kind::Button,
            pressed: false,
            disabled: false,
        }
    }

    /// A toggle, on or off.
    pub fn toggle(key: impl Into<String>, label: impl Into<String>, pressed: bool) -> Self {
        Self {
            kind: Kind::Toggle,
            pressed,
            ..Self::button(key, label)
        }
    }

    /// A separator.
    pub fn separator() -> Self {
        Self {
            kind: Kind::Separator,
            ..Self::button("", "")
        }
    }

    /// The same item, disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    fn skipped(&self) -> bool {
        self.disabled || self.kind == Kind::Separator
    }
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the bar.
    Ignored,
    /// The cursor moved to this item.
    Moved(usize),
    /// The item was pressed (a toggle has already turned).
    Pressed(usize),
}

/// The items and the cursor.
#[derive(Clone, Debug)]
pub struct State {
    items: Vec<Item>,
    cursor: Option<usize>,
    axis: Axis,
}

impl State {
    /// The cursor starts on the first item that can be reached.
    pub fn new(items: Vec<Item>, axis: Axis) -> Self {
        let cursor = items.iter().position(|i| !i.skipped());
        Self {
            items,
            cursor,
            axis,
        }
    }

    /// The items.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The way the bar runs.
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// The way the bar runs.
    pub fn set_axis(&mut self, axis: Axis) {
        self.axis = axis;
    }

    /// The index the cursor is on.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// Press the item with this key (the pointer's way, and the cursor goes there); `None` when there is no such item or it cannot
    /// be pressed.
    pub fn press_key(&mut self, key: &str) -> Option<usize> {
        let i = self
            .items
            .iter()
            .position(|item| item.key == key && !item.skipped())?;
        self.cursor = Some(i);
        Some(self.fire(i))
    }

    fn fire(&mut self, i: usize) -> usize {
        if self.items[i].kind == Kind::Toggle {
            self.items[i].pressed = !self.items[i].pressed;
        }
        i
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        if matches!(key, Key::Enter | Key::Space | Key::Activate) {
            return match self.cursor {
                Some(i) if !self.items[i].skipped() => Outcome::Pressed(self.fire(i)),
                _ => Outcome::Ignored,
            };
        }
        let items = &self.items;
        match next(
            items.len(),
            self.cursor,
            key,
            self.axis,
            false,
            &|i| items[i].skipped(),
        ) {
            Some(to) => {
                self.cursor = Some(to);
                Outcome::Moved(to)
            }
            None => Outcome::Ignored,
        }
    }
}
