//! The words of the help. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The texts of the help; the help itself (the blocks) is the program's.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The button to the documentation.
    pub open_docs: String,
    /// The language the words were made for; the sentences with a name are formatted in it.
    pub locale: Locale,
}

/// The two language files of the help; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The help's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        Self {
            open_docs: Mf2Texts::new(TEXTS, locale).plain("help-open-docs"),
            locale: locale.clone(),
        }
    }
}

impl Labels {
    fn say(&self, id: &str, args: &Args) -> String {
        Mf2Texts::new(TEXTS, &self.locale)
            .say(&TextId::new(id), args)
            .text()
            .to_owned()
    }

    /// "There is no help for "versions" yet."
    pub fn missing(&self, topic: &str) -> String {
        self.say("help-missing", &Args::new().text("topic", topic))
    }

    /// The sentence for a source that does not answer.
    pub fn unavailable(&self) -> String {
        Mf2Texts::new(TEXTS, &self.locale).plain("help-unavailable")
    }

    /// "Shown in English: not translated into your language yet."
    pub fn shown_in(&self, language: &str) -> String {
        self.say("help-shown-in", &Args::new().text("language", language))
    }

    /// The note under a translation that is older than its original.
    pub fn outdated(&self) -> String {
        Mf2Texts::new(TEXTS, &self.locale).plain("help-outdated")
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
