//! What the input does with its parts: read them, put one in, move the keyboard, and the work after a change.
//!
//! Pattern: none — the methods of `Inner` (the state of `FieldInput`), kept apart so `input.rs` stays the builder.

use std::sync::atomic::Ordering;

use super::input::Inner;
use crate::field::Entry;
use crate::kinds::stepped;

impl Inner {
    /// How many parts the input has.
    pub(super) fn count(&self) -> usize {
        if self.kind.field().entry() == Entry::Switch {
            return 1;
        }
        self.refs.len().max(self.area_refs.len())
    }

    /// Put `text` into part `j`, as the person would have typed it.
    pub(super) fn set_part(&self, j: usize, text: &str) {
        if self.kind.field().entry() == Entry::Switch {
            *self
                .flag
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = text.to_owned();
            return;
        }
        match self.area_refs.get(j) {
            Some(area) => area.set_value(text),
            None => self.refs[j].set_value(text),
        }
    }

    /// The keyboard to part `j`.
    pub(super) fn focus_part(&self, j: usize) {
        if self.kind.field().entry() == Entry::Switch {
            return;
        }
        match self.area_refs.get(j) {
            Some(area) => area.focus(),
            None => self.refs[j].focus(),
        }
    }

    pub(super) fn parts(&self) -> Vec<String> {
        if self.kind.field().entry() == Entry::Switch {
            return vec![
                self.flag
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ];
        }
        if !self.areas.is_empty() {
            return self
                .areas
                .iter()
                .map(|a| {
                    a.lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .value()
                })
                .collect();
        }
        self.data
            .iter()
            .map(|d| {
                d.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .value
                    .clone()
            })
            .collect()
    }

    /// Part `i` changed: spread a whole that was typed into it, remember it, ask the question
    /// again, tell the owner.
    pub(super) fn changed(&self, i: usize) {
        let field = self.kind.field();
        let typed = self.parts();
        let settled = field.settle(&typed);
        for (j, part) in settled.iter().enumerate().take(self.count()) {
            if typed.get(j) != Some(part) {
                self.set_part(j, part);
            }
        }
        let mut at = i;
        // Part `i` gave up text to a later one (an `@` typed into the local part): go on there.
        if settled.get(i) != typed.get(i) && i + 1 < self.count() {
            let last = (i + 1..settled.len().min(self.count()))
                .rev()
                .find(|j| typed.get(*j) != settled.get(*j))
                .unwrap_or(i + 1);
            self.focus_part(last);
            at = last;
        }
        self.touched.store(true, Ordering::Relaxed);
        self.last.store(at, Ordering::Relaxed);
        self.live.refresh_now();
        if let Some(f) = &self.on_change {
            f(&field.join(&settled));
        }
    }

    /// The person moved the number in part `i` by `delta` steps (the arrow keys): the typed text is the start, nothing typed
    /// starts at 0; text that is not a number is left as it is.
    pub(super) fn step(&self, i: usize, signed: bool, delta: i32) {
        if let Some(next) = stepped(signed, &self.parts()[i], delta) {
            self.set_part(i, &next);
            self.changed(i);
        }
    }

    /// The switch of a yes/no type was flipped by the person.
    pub(super) fn answered(&self, yes: bool) {
        self.set_part(0, if yes { "true" } else { "false" });
        self.changed(0);
    }
}
