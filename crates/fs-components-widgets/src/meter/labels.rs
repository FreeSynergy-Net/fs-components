//! The words of the meter. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

use super::zone::Zone;

/// The texts of the meter.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Said for the good zone.
    pub good: String,
    /// Said for the fair zone.
    pub fair: String,
    /// Said for the poor zone.
    pub poor: String,
}

impl Labels {
    /// The word for a zone.
    pub fn of(&self, zone: Zone) -> &str {
        match zone {
            Zone::Good => &self.good,
            Zone::Fair => &self.fair,
            Zone::Poor => &self.poor,
        }
    }
}

/// The two language files of the meter; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The meter's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            good: t.plain("meter-good"),
            fair: t.plain("meter-fair"),
            poor: t.plain("meter-poor"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
