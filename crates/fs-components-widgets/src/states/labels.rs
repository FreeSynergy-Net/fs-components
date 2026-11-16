//! The standard words of the state surfaces. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The words of the failed, loading and partial surfaces. The words of the empty surface are the caller's (they say what is
/// missing: "No programs", "No appointments").
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The retry button.
    pub retry: String,
    /// What a screen reader hears over the placeholder.
    pub loading: String,
    /// The title of the failed surface.
    pub failed: String,
    /// The language the words were made for; the partial sentence is formatted in it.
    pub locale: Locale,
}

/// The two language files of the surfaces; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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

impl Localized for Labels {
    /// The surfaces' words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            retry: t.plain("state-retry"),
            loading: t.plain("state-loading"),
            failed: t.plain("state-failed"),
            locale: locale.clone(),
        }
    }
}

impl Labels {
    /// "2 of 9 could not be shown".
    pub fn partial(&self, failed: usize, total: usize) -> String {
        Mf2Texts::new(TEXTS, &self.locale)
            .say(
                &TextId::new("state-partial"),
                &Args::new()
                    .count("failed", failed as u64)
                    .count("total", total as u64),
            )
            .text()
            .to_owned()
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
