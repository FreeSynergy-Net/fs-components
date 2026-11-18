//! The reader's sentences of the adapter: German and English from `MessageFormat` 2 files (ADR 0057), English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded files, as the board's own words.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

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

/// The sentences.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordsTexts {
    /// A field the reader has no right to see.
    pub hidden_no_right: String,
    /// A field marked sensitive and not shared with her.
    pub hidden_sensitive: String,
    /// Writing is not possible yet (the adapter reads only).
    pub write_later: String,
    /// A record taken over by someone.
    pub lock_claimed: String,
    /// A record she may read but not change.
    pub lock_read_only: String,
    /// A record an admin locked.
    pub lock_admin: String,
    /// A page was refused (a hidden value came with it).
    pub page_refused: String,
    /// The data could not be asked for; the last state is still shown.
    pub stale: String,
    /// Not everything was read (more pages than allowed).
    pub truncated: String,
    /// A record lies in no group of the grouping and is not shown.
    pub group_unknown: String,
}

impl Localized for RecordsTexts {
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            hidden_no_right: t.plain("records-hidden-no-right"),
            hidden_sensitive: t.plain("records-hidden-sensitive"),
            write_later: t.plain("records-write-later"),
            lock_claimed: t.plain("records-lock-claimed"),
            lock_read_only: t.plain("records-lock-read-only"),
            lock_admin: t.plain("records-lock-admin"),
            page_refused: t.plain("records-page-refused"),
            stale: t.plain("records-stale"),
            truncated: t.plain("records-truncated"),
            group_unknown: t.plain("records-group-unknown"),
        }
    }
}

impl Default for RecordsTexts {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
