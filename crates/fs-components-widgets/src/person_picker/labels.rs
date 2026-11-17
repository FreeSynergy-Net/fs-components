//! The words of the person picker. They come from the program; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.
//! An error is said by code and parameter: the sentence of the code, then the code itself, so a report can name it.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

use super::model::{ErrorCode, PickerError};
use crate::states;

/// The words of the picker.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The heading.
    pub title: String,
    /// Placeholder of the search field.
    pub search: String,
    /// Button: confirm.
    pub confirm: String,
    /// Button: cancel.
    pub cancel: String,
    /// Button: assign to me.
    pub me: String,
    /// No candidate fits the query.
    pub none_found: String,
    /// The source has nobody at all.
    pub nothing_offered: String,
    /// A person whose name the source does not release.
    pub unnamed: String,
    /// "Assign to me" without an own card.
    pub me_unset: String,
    /// More people exist than are shown (the source says only that, not how many).
    pub more: String,
    /// The words of the loading and retry surfaces.
    pub states: states::Labels,
    /// The language the words were made for.
    pub locale: Locale,
}

/// The two language files of the picker; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The picker's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            title: t.plain("picker-title"),
            search: t.plain("picker-search"),
            confirm: t.plain("picker-confirm"),
            cancel: t.plain("picker-cancel"),
            me: t.plain("picker-me"),
            none_found: t.plain("picker-none-found"),
            nothing_offered: t.plain("picker-nothing-offered"),
            unnamed: t.plain("picker-unnamed"),
            me_unset: t.plain("picker-me-unset"),
            more: t.plain("picker-more"),
            states: states::Labels::for_locale(locale),
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

    /// "3 chosen".
    pub fn chosen(&self, count: usize) -> String {
        self.say("picker-chosen", &Args::new().count("count", count as u64))
    }

    /// The sentence of an error with its parameter, then the code in brackets: "... [picker.denied]".
    pub fn error(&self, error: &PickerError) -> String {
        let id = match error.code {
            ErrorCode::SourceMissing => "picker-error-source-missing",
            ErrorCode::Denied => "picker-error-denied",
            ErrorCode::Unavailable => "picker-error-unavailable",
            _ => "picker-error-failed",
        };
        let sentence = self.say(id, &Args::new().text("param", error.param.clone()));
        format!("{sentence} [{}]", error.code.code())
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
