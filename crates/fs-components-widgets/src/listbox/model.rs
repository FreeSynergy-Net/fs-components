//! The pure state of a listbox.
//!
//! Pattern: State — the options, the cursor, the picks, what a key does; no element, no callback.

use crate::keys::{Key, KeyEvent};
use crate::roving::{Axis, next};

/// One option.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opt {
    /// What the caller gets back; part of the id (`<id>-opt-<key>`).
    pub key: String,
    /// What the person reads.
    pub label: String,
    /// Cannot be picked; the keys skip it.
    pub disabled: bool,
}

impl Opt {
    /// An option that can be picked.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// The same option, disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the list.
    Ignored,
    /// The cursor moved; the picks are as they were.
    Moved(usize),
    /// The picks changed (the option the cursor is on, or all of them).
    Picked,
}

/// The options, the cursor and the picks.
#[derive(Clone, Debug)]
pub struct State {
    opts: Vec<Opt>,
    cursor: Option<usize>,
    picked: Vec<bool>,
    multi: bool,
}

impl State {
    /// In the single way the first option that can be picked is picked; in the several way nothing is.
    pub fn new(opts: Vec<Opt>, multi: bool) -> Self {
        let cursor = opts.iter().position(|o| !o.disabled);
        let mut picked = vec![false; opts.len()];
        if let (false, Some(i)) = (multi, cursor) {
            picked[i] = true;
        }
        Self {
            opts,
            cursor,
            picked,
            multi,
        }
    }

    /// The options.
    pub fn opts(&self) -> &[Opt] {
        &self.opts
    }

    /// Whether several may be picked.
    pub fn multi(&self) -> bool {
        self.multi
    }

    /// The index the cursor is on.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// Whether option `i` is picked.
    pub fn is_picked(&self, i: usize) -> bool {
        self.picked.get(i).copied().unwrap_or(false)
    }

    /// The keys of the picked options, in list order.
    pub fn picked_keys(&self) -> Vec<String> {
        self.opts
            .iter()
            .zip(&self.picked)
            .filter(|(_, p)| **p)
            .map(|(o, _)| o.key.clone())
            .collect()
    }

    /// Pick (or unpick, in the several way) the option with this key, from the pointer or the program; the cursor goes there.
    /// `false` when there is no such option or it is disabled.
    pub fn pick_key(&mut self, key: &str) -> bool {
        match self.opts.iter().position(|o| o.key == key && !o.disabled) {
            Some(i) => {
                self.cursor = Some(i);
                self.pick(i);
                true
            }
            None => false,
        }
    }

    fn pick(&mut self, i: usize) {
        if self.multi {
            self.picked[i] = !self.picked[i];
        } else {
            self.picked.iter_mut().for_each(|p| *p = false);
            self.picked[i] = true;
        }
    }

    /// What a key does.
    pub fn press(&mut self, event: KeyEvent) -> Outcome {
        match event.key {
            Key::Enter | Key::Space | Key::Activate => match self.cursor {
                Some(i) if self.multi => {
                    self.pick(i);
                    Outcome::Picked
                }
                _ => Outcome::Ignored,
            },
            Key::Other(65) if event.ctrl && self.multi => {
                for (p, o) in self.picked.iter_mut().zip(&self.opts) {
                    *p = !o.disabled;
                }
                Outcome::Picked
            }
            key => {
                let opts = &self.opts;
                let Some(to) = next(
                    opts.len(),
                    self.cursor,
                    key,
                    Axis::Vertical,
                    false,
                    &|i| opts[i].disabled,
                ) else {
                    return Outcome::Ignored;
                };
                self.cursor = Some(to);
                if self.multi || self.picked[to] {
                    return Outcome::Moved(to);
                }
                self.pick(to);
                Outcome::Picked
            }
        }
    }
}
