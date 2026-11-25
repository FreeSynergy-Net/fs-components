//! Readable words for the labels of the grid when the host has no sentence for a key.
//!
//! Pattern: Decorator over [`Say`] — the host's answer wins; when it answers with the key itself (it has no sentence, the
//! rule of `said_or`), the English table of `fs-components-words` answers, so a button or a day name never shows
//! `calendar.previous` or `weekday.0`. The table is the built-in last resort of that crate (`English`), nothing is
//! invented here.

use std::sync::{Arc, OnceLock};

use fs_components_fields::Say;
use fs_components_words::{Args, English, TextId, Words};

/// Keys of the grid and their English words: the toolbar, the modes, the weekdays from Monday (0) and the months from 1.
const TABLE: &[(&str, &str)] = &[
    ("calendar.previous", "Previous"),
    ("calendar.today", "Today"),
    ("calendar.next", "Next"),
    ("calendar.week-short", "Wk"),
    ("calendar.mode.month", "Month"),
    ("calendar.mode.week", "Week"),
    ("calendar.mode.day", "Day"),
    ("calendar.weekday.0", "Mon"),
    ("calendar.weekday.1", "Tue"),
    ("calendar.weekday.2", "Wed"),
    ("calendar.weekday.3", "Thu"),
    ("calendar.weekday.4", "Fri"),
    ("calendar.weekday.5", "Sat"),
    ("calendar.weekday.6", "Sun"),
    ("calendar.month.1", "January"),
    ("calendar.month.2", "February"),
    ("calendar.month.3", "March"),
    ("calendar.month.4", "April"),
    ("calendar.month.5", "May"),
    ("calendar.month.6", "June"),
    ("calendar.month.7", "July"),
    ("calendar.month.8", "August"),
    ("calendar.month.9", "September"),
    ("calendar.month.10", "October"),
    ("calendar.month.11", "November"),
    ("calendar.month.12", "December"),
];

fn english() -> &'static English {
    static ENGLISH: OnceLock<English> = OnceLock::new();
    ENGLISH.get_or_init(|| English::new(TABLE))
}

/// A [`Say`] that falls back to the English word of the grid for a key the inner one cannot answer.
pub struct Readable(Arc<dyn Say>);

impl Readable {
    /// `inner` first, the English table second.
    pub fn new(inner: Arc<dyn Say>) -> Self {
        Self(inner)
    }
}

impl Say for Readable {
    fn say(&self, key: &str) -> String {
        let said = self.0.say(key);
        if said != key {
            return said;
        }
        let word = english().say(&TextId::new(key), &Args::new());
        if word.is_found() {
            word.text().to_owned()
        } else {
            said
        }
    }
}
