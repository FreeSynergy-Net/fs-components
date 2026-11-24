//! The words of the timeline in German and English, as the default for a program that has no catalog of its own.
//!
//! Pattern: Adapter — embedded `MessageFormat` 2 files (`texts/en.mf2`, `texts/de.mf2`, ADR 0057) read by `fs-components-mf2`,
//! answering the dotted keys of [`Say`] (`timeline.zoom.day`) from the ids with dashes (`timeline-zoom-day`). A program with
//! its own catalog hands its own `Say` to the timeline instead.

use fs_components_fields::Say;
use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::Locale;

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

/// The words of the timeline in one language; a language without a file comes in English.
pub struct TimelineTexts(Mf2Texts);

impl TimelineTexts {
    /// The words in `locale`.
    pub fn new(locale: &Locale) -> Self {
        Self(Mf2Texts::new(TEXTS, locale))
    }
}

impl Say for TimelineTexts {
    fn say(&self, key: &str) -> String {
        self.0.plain(&key.replace('.', "-"))
    }
}
