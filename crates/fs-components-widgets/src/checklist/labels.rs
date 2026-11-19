//! The words of the checklist. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The texts of the checklist.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The hint in the line for a new item.
    pub add: String,
    /// The first word of the name of a remove button ("Remove" + the item's text).
    pub remove: String,
}

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

impl Localized for Labels {
    /// The checklist's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            add: t.plain("checklist-add"),
            remove: t.plain("checklist-remove"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
