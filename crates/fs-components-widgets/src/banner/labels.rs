//! The words of the banner. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The texts of the banner. What it says is the caller's.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The name of the dismiss mark.
    pub close: String,
}

/// The two language files of the banner; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The banner's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        Self {
            close: Mf2Texts::new(TEXTS, locale).plain("banner-close"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
