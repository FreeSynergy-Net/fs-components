//! The words of the carousel. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The texts of the carousel.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// `aria-roledescription` of the root.
    pub kind: String,
    /// `aria-roledescription` of a slide.
    pub slide: String,
    /// The name of the previous button.
    pub previous: String,
    /// The name of the next button.
    pub next: String,
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
    /// The carousel's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            kind: t.plain("carousel-kind"),
            slide: t.plain("carousel-slide"),
            previous: t.plain("carousel-previous"),
            next: t.plain("carousel-next"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
