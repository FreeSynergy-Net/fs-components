//! `DateData`: where the date data of a language comes from, and which date functions a language gets.
//!
//! Pattern: Strategy (Provider) — `DateData` hands out the date data (an ICU4X blob) of one language, or none; the texts
//! do not know where it lives. There is no list of languages in the code: the data is a package per language, loaded when
//! it is installed (ADR 0077). `NoDates` is the empty provider (tests, and the fallback when no package is there).
//!
//! The fallback is fixed here: a language without usable data does NOT lose its messages; only the date or time value is
//! shown neutrally (`2026-11-26 12:05`, the `mf2` neutral formatter). ICU4X itself is not connected yet (the next step
//! turns it on); until then every language gets the neutral functions, whatever `blob` returns.

use std::sync::Arc;

use fs_components_words::Locale;
use mf2::fn_datetime::{DateTimeFunction, Neutral};
use mf2::{Function, Registry};

/// The date data (an ICU4X blob) of a language, from wherever the installed packages keep it.
pub trait DateData: Send + Sync {
    /// The blob for `tag`, or `None` when no package has it. A blob that cannot be used is no error either: the texts
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

/// The functions a language formats with, given its date data. ICU4X is not connected yet: a missing blob, a valid one and
/// a broken one all give the neutral registry (the message stays, only the date value is neutral).
pub(crate) fn registry_for(_blob: Option<&[u8]>) -> &'static Registry {
    &NEUTRAL
}
