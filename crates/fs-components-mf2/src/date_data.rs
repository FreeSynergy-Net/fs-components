//! `DateData`: where the date data of a language comes from, and which date functions a language gets.
//!
//! Pattern: Strategy (Provider) — `DateData` hands out the date data (the ICU4X date table the packer writes, see
//! `fs-release pack-texts`) of one language, or none; the texts do not know where it lives. There is no list of languages in
//! the code: the data is a package per language, loaded when it is installed (ADR 0077). `NoDates` is the empty provider
//! (tests, and the fallback when no package is there); `DateDir` reads the table of an installed package.
//!
//! The fallback is fixed here: a language without usable data does NOT lose its messages; only the date or time value is
//! shown neutrally (`2026-11-26 12:05`, the `mf2` neutral formatter). With a usable table `:datetime`, `:date` and `:time`
//! format by the language (`26.11.2026, 12:05`); see `icu_dates.rs` for how a table is checked and attached.

use std::path::PathBuf;
use std::sync::Arc;

use fs_components_words::Locale;
use mf2::fn_datetime::{DateTimeFunction, Neutral};
use mf2::{Function, Registry};

/// The date data (the ICU4X table of the packer) of a language, from wherever the installed packages keep it.
pub trait DateData: Send + Sync {
    /// The table for `tag`, or `None` when no package has it. A table that cannot be used is no error either: the texts
    /// fall back to the neutral display, and the message stays.
    fn blob(&self, tag: &Locale) -> Option<Arc<[u8]>>;
}

/// No date data at all: every date or time is shown neutrally.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDates;

impl DateData for NoDates {
    fn blob(&self, _tag: &Locale) -> Option<Arc<[u8]>> {
        None
    }
}

/// The date data of the installed packages: `<root>/dates/<tag>/icu.table`, one directory per language. No list of languages
/// here; a language is there when its directory is.
#[derive(Clone, Debug)]
pub struct DateDir {
    root: PathBuf,
}

impl DateDir {
    /// The packages below `root` (the directory that holds `dates/`).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl DateData for DateDir {
    fn blob(&self, tag: &Locale) -> Option<Arc<[u8]>> {
        // A `Locale` is letters, digits and `-` only, so the tag cannot leave the directory.
        let path = self.root.join("dates").join(tag.as_str()).join("icu.table");
        std::fs::read(path).ok().map(Arc::from)
    }
}

static DATETIME: DateTimeFunction<Neutral> = DateTimeFunction::datetime(Neutral);
static DATE: DateTimeFunction<Neutral> = DateTimeFunction::date(Neutral);
static TIME: DateTimeFunction<Neutral> = DateTimeFunction::time(Neutral);
static FUNCTIONS: [(&str, &dyn Function); 8] = [
    ("string", &mf2::functions::STRING),
    ("number", &mf2::fn_number::NUMBER),
    ("integer", &mf2::fn_number::INTEGER),
    ("percent", &mf2::fn_number::PERCENT),
    ("offset", &mf2::fn_number::OFFSET),
    ("datetime", &DATETIME),
    ("date", &DATE),
    ("time", &TIME),
];
/// Text functions, numbers, and the neutral date functions.
static NEUTRAL: Registry = Registry::new(&FUNCTIONS).with_numbers(&mf2::fn_number::NUMBERS);

/// The ICU4X date functions of the packed tables: Gregorian, no time zones (what the packer writes; the standard `:datetime`
/// of `mf2` would want any calendar and zones and fail on our table).
mod icu {
    mf2::__date_statics!(gregorian no_zones);
}
static ICU_FUNCTIONS: [(&str, &dyn Function); 8] = [
    ("string", &mf2::functions::STRING),
    ("number", &mf2::fn_number::NUMBER),
    ("integer", &mf2::fn_number::INTEGER),
    ("percent", &mf2::fn_number::PERCENT),
    ("offset", &mf2::fn_number::OFFSET),
    ("datetime", &icu::DATETIME),
    ("date", &icu::DATE),
    ("time", &icu::TIME),
];
static ICU: Registry = Registry::new(&ICU_FUNCTIONS).with_numbers(&mf2::fn_number::NUMBERS);

/// The functions a language formats with: the ICU4X ones when its catalog carries a usable date table, else the neutral ones
/// (the message stays, only the date value is neutral).
pub(crate) fn registry_for(with_table: bool) -> &'static Registry {
    if with_table { &ICU } else { &NEUTRAL }
}
