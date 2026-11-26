//! The words of the calendar grid: a catalog in English and German, and the chain that asks the host first.
//!
//! Pattern: Chain of Responsibility — the sentence of the host (`ctx.say`) answers first; when it answers with the key itself
//! (it has no sentence, the rule of `said_or`), the catalog of the calendar components (`texts/*.mf2`, ADR 0057, read by
//! `fs-components-mf2`) answers; when that has none either, the key stays. There is no table of words in code. The keys are the
//! ids of the catalog (`calendar-view-day`); the names of weekdays and months are not here, they are the CLDR names of
//! `DrawCtx.dates` (`fs_components_words::DateNames`).

use std::sync::Arc;

use fs_components_fields::Say;
use fs_components_mf2::{Mf2Texts, Source, check_pairs};
use fs_components_words::{Args, Locale, TextId, Words};

/// The two language files; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("texts/en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("texts/de.mf2"),
    },
];

/// The ids the grid asks for (a test holds them equal to the catalog).
pub const KEYS: &[&str] = &[
    "calendar-view-month",
    "calendar-view-week",
    "calendar-view-day",
    "calendar-view-agenda",
    "calendar-today",
    "calendar-previous",
    "calendar-next",
    "calendar-week-short",
    "calendar-all-day",
    "calendar-empty",
    "calendar-more",
    "calendar-colour-unreadable",
    "calendar-colour-faint",
];

/// The id of the button or title of a mode: `calendar-view-<name>`.
pub fn view_key(name: &str) -> String {
    format!("calendar-view-{name}")
}

/// Whether the shipped catalogs pair up (the same ids in both languages).
pub fn pairs_missing() -> Vec<String> {
    check_pairs(TEXTS)
}

/// The words of the calendar components in one language; a language without a file comes in English.
pub struct CalendarTexts(Mf2Texts);

impl CalendarTexts {
    /// The words in `locale`.
    pub fn new(locale: &Locale) -> Self {
        Self(Mf2Texts::new(TEXTS, locale))
    }

    /// The sentence `id` with the number `n` in it.
    pub fn counted(&self, id: &str, n: usize) -> String {
        let args = Args::new().count("n", n as u64);
        self.0.say(&TextId::new(id), &args).text().to_owned()
    }

    /// The mark of `n` more items than the cell shows (`+3`).
    pub fn more(&self, n: usize) -> String {
        self.counted("calendar-more", n)
    }
}

impl Say for CalendarTexts {
    fn say(&self, key: &str) -> String {
        self.0.plain(key)
    }
}

/// The host's words first, the catalog second, the key last.
pub struct Chain {
    host: Arc<dyn Say>,
    catalog: CalendarTexts,
}

impl Chain {
    /// `host` first, then the catalog in `locale`.
    pub fn new(host: Arc<dyn Say>, locale: &Locale) -> Self {
        Self {
            host,
            catalog: CalendarTexts::new(locale),
        }
    }

    /// The words for `key`: the host's, else the catalog's, else the key.
    pub fn say(&self, key: &str) -> String {
        let said = self.host.say(key);
        if said == key {
            self.catalog.say(key)
        } else {
            said
        }
    }

    /// The mark `+n` (the catalog's: a number needs no sentence of the host).
    pub fn more(&self, n: usize) -> String {
        self.catalog.more(n)
    }
}

impl Say for Chain {
    fn say(&self, key: &str) -> String {
        Chain::say(self, key)
    }
}
