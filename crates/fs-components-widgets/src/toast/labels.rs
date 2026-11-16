//! The words of the toasts. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The texts of the toasts. The message itself is the caller's.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The name of the close mark.
    pub close: String,
    /// The language the words were made for; the count of waiting toasts is formatted in it.
    pub locale: Locale,
}

/// The two language files of the toasts; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The toasts' words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        Self {
            close: Mf2Texts::new(TEXTS, locale).plain("toast-close"),
            locale: locale.clone(),
        }
    }
}

impl Labels {
    /// "3 more messages": how many older toasts wait behind the ones in view.
    pub(super) fn more(&self, count: usize) -> String {
        Mf2Texts::new(TEXTS, &self.locale)
            .say(
                &TextId::new("toast-more"),
                &Args::new().count("count", count as u64),
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
