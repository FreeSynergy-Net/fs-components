//! The standard words of the dialogs. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The words of the buttons of every dialog of this family. A dialog may name its own buttons ("Install", "Not now").
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Button: take the choice.
    pub ok: String,
    /// Button: leave without changing anything.
    pub cancel: String,
    /// Button: agree (the licence).
    pub accept: String,
    /// Button: do not agree.
    pub decline: String,
    /// Button: hand the text over to the clipboard.
    pub copy: String,
    /// The name of the close mark.
    pub close: String,
}

/// The two language files of the dialogs; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The dialogs' words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            ok: t.plain("dialog-ok"),
            cancel: t.plain("dialog-cancel"),
            accept: t.plain("dialog-accept"),
            decline: t.plain("dialog-decline"),
            copy: t.plain("dialog-copy"),
            close: t.plain("dialog-close"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
