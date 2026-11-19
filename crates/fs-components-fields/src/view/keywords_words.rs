//! The words of the `keywords` input: making a chip, refusing a word, removing one, reading what the line says.
//!
//! Pattern: Facade — the methods of [`KeywordsInput`] that change the words, kept apart from the drawing in `keywords.rs`.

use std::sync::PoisonError;
use std::sync::atomic::Ordering;

use fs_components_widgets::chip::Chip;

use super::keywords::{KeywordsInput, MOST, made};
use crate::Problem;
use crate::kinds::canonical_keywords;

impl KeywordsInput {
    /// Start with this stored (or typed) list.
    #[must_use]
    pub fn value(self, text: &str) -> Self {
        for word in text.split(',') {
            let _ = self.add_word(word, false);
        }
        self
    }

    /// The words as chips now, in the order they were made.
    pub fn words(&self) -> Vec<String> {
        self.inner
            .words
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The stored text now (canonical: sorted, "a, b, c"; empty if none).
    pub fn text(&self) -> String {
        let words = self.words().join(", ");
        canonical_keywords(&words).unwrap_or_default()
    }

    /// The last thing that was refused (a word that is too long, too many words), `None` if all is well.
    pub fn problem(&self) -> Option<Problem> {
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Turn `raw` into a chip (as a comma or Enter does); false when nothing was added (blank, a word already there, refused).
    pub fn add_word(&self, raw: &str, touched: bool) -> bool {
        if touched {
            self.inner.touched.store(true, Ordering::Relaxed);
        }
        let word = match made(raw) {
            Ok(Some(w)) => w,
            Ok(None) => return false,
            Err(p) => return self.refuse(p),
        };
        {
            let mut words = self
                .inner
                .words
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if words.contains(&word) {
                return false;
            }
            if words.len() >= MOST {
                drop(words);
                return self.refuse(Problem::TooMany);
            }
            words.push(word.clone());
        }
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
        self.inner.chips.add(Chip::new(word.clone(), word));
        self.changed();
        true
    }

    pub(super) fn refuse(&self, p: Problem) -> bool {
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(p);
        self.inner.live.refresh_now();
        false
    }

    /// Remove the word (the chip's button, Delete on the row, Backspace on the empty line); false when it is not there.
    pub fn remove_word(&self, word: &str) -> bool {
        let found = {
            let mut words = self
                .inner
                .words
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            words
                .iter()
                .position(|w| w == word)
                .map(|i| words.remove(i))
                .is_some()
        };
        if found {
            self.inner.chips.remove(word);
            self.inner.touched.store(true, Ordering::Relaxed);
            self.changed();
        }
        found
    }

    pub(super) fn changed(&self) {
        self.inner.live.refresh_now();
        let f = self
            .inner
            .on_change
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(f) = f {
            f(&self.text());
        }
    }

    /// The line changed: every word before a comma becomes a chip, what follows the last comma stays typed.
    pub(super) fn typed(&self, now: &str) {
        let Some((done, rest)) = now.rsplit_once(',') else {
            return;
        };
        for word in done.split(',') {
            let _ = self.add_word(word, true);
        }
        self.inner.line.set_value(rest.trim_start().to_owned());
    }

    pub(super) fn submitted(&self) {
        let now = self
            .inner
            .data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .value
            .clone();
        if self.add_word(&now, true) || now.trim().is_empty() {
            self.inner.line.set_value(String::new());
        }
    }

    pub(super) fn backspace_empty(&self) {
        if let Some(last) = self.words().last().cloned() {
            self.remove_word(&last);
        }
    }
}
