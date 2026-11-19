//! The pure state of a row of tabs.
//!
//! Pattern: State — which tab is chosen, where the cursor is, what a key does; no element, no callback.

use crate::keys::Key;
use crate::roving::{Axis, next};

/// One tab.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    /// What the caller gets back; also part of the ids (`<id>-tab-<key>`, `<id>-panel-<key>`).
    pub key: String,
    /// What the person reads.
    pub label: String,
    /// Cannot be chosen; the keys skip it.
    pub disabled: bool,
}

impl Tab {
    /// A tab that can be chosen.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// The same tab, disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the tabs.
    Ignored,
    /// The cursor moved to this tab; the choice stays.
    Moved(usize),
    /// This tab is chosen now.
    Chosen(usize),
}

/// The tabs, the chosen one and the cursor.
#[derive(Clone, Debug)]
pub struct State {
    tabs: Vec<Tab>,
    chosen: Option<usize>,
    cursor: Option<usize>,
    manual: bool,
    axis: Axis,
}

impl State {
    /// The first tab that is not disabled is chosen.
    pub fn new(tabs: Vec<Tab>, manual: bool, axis: Axis) -> Self {
        let chosen = tabs.iter().position(|t| !t.disabled);
        Self {
            tabs,
            chosen,
            cursor: chosen,
            manual,
            axis,
        }
    }

    /// Manual (the cursor moves, Enter or Space chooses) or automatic (the choice follows the cursor).
    pub fn set_manual(&mut self, manual: bool) {
        self.manual = manual;
    }

    /// The way the row runs.
    pub fn set_axis(&mut self, axis: Axis) {
        self.axis = axis;
    }

    /// The tabs.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// The way the row runs.
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// The index of the chosen tab.
    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    /// The index the cursor is on (the same as the choice unless the manual way moved it away).
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// The key of the chosen tab.
    pub fn chosen_key(&self) -> Option<&str> {
        self.chosen.map(|i| self.tabs[i].key.as_str())
    }

    /// Choose the tab with this key; `false` when there is none or it is disabled. The cursor goes with it.
    pub fn choose(&mut self, key: &str) -> bool {
        match self.tabs.iter().position(|t| t.key == key && !t.disabled) {
            Some(i) => {
                self.chosen = Some(i);
                self.cursor = Some(i);
                true
            }
            None => false,
        }
    }

    /// What a key does: arrows, Home and End move the cursor (and in the automatic way the choice with it); Enter, Space
    /// (`Activate`) choose the cursor tab in the manual way.
    pub fn press(&mut self, key: Key) -> Outcome {
        if matches!(key, Key::Enter | Key::Space | Key::Activate) {
            return match self.cursor {
                Some(i) if self.manual && self.chosen != Some(i) => {
                    self.chosen = Some(i);
                    Outcome::Chosen(i)
                }
                _ => Outcome::Ignored,
            };
        }
        let tabs = &self.tabs;
        let Some(to) = next(tabs.len(), self.cursor, key, self.axis, true, &|i| {
            tabs[i].disabled
        }) else {
            return Outcome::Ignored;
        };
        self.cursor = Some(to);
        if self.manual || self.chosen == Some(to) {
            return Outcome::Moved(to);
        }
        self.chosen = Some(to);
        Outcome::Chosen(to)
    }
}
