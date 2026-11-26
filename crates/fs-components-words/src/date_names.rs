//! `DateNames`: the names of calendar fields in the language of the reader.
//!
//! Pattern: Strategy (Provider) – the CLDR names of calendar fields in the reader's language; ICU4X
//! behind it (fs-components-mf2 `IcuDateNames`), a Null Object without data (`NeutralDateNames`).
//! This file holds only the interface; there is no implementation and no language table here.

/// A day of the week, numbered as in ISO 8601 (Monday = 1 … Sunday = 7).
///
/// Pattern: Value Object – a closed set with a stable number, so a caller never passes a bare
/// integer for a weekday.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Weekday {
    /// Monday, ISO 8601 day 1.
    Monday = 1,
    /// Tuesday, ISO 8601 day 2.
    Tuesday = 2,
    /// Wednesday, ISO 8601 day 3.
    Wednesday = 3,
    /// Thursday, ISO 8601 day 4.
    Thursday = 4,
    /// Friday, ISO 8601 day 5.
    Friday = 5,
    /// Saturday, ISO 8601 day 6.
    Saturday = 6,
    /// Sunday, ISO 8601 day 7.
    Sunday = 7,
}

/// How long a name is, the same three steps as ICU4X `Length`.
///
/// Pattern: Value Object – ordered from the longest to the shortest form (`Long` < `Medium` <
/// `Short`), nothing of our own added to what CLDR offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Width {
    /// The full name ("Monday", "November").
    Long,
    /// The abbreviated name ("Mon", "Nov").
    Medium,
    /// The shortest name ("Mo", "N"); may be ambiguous.
    Short,
}

/// The names of weekdays, months and dates of one language, and the first day of its week.
///
/// Pattern: Strategy (Provider) – calendar views ask this interface and never hold a table of
/// names themselves; ICU4X, a data package or a neutral fallback sit behind it. Month numbers run
/// 1 (January) to 12 (December), days from 1. A value that does not exist never panics: the
/// implementation returns a neutral text instead of an empty one.
pub trait DateNames: Send + Sync {
    /// Strategy: the name of a weekday in the given width (ICU4X field set `E`).
    fn weekday(&self, day: Weekday, width: Width) -> String;

    /// Strategy: the stand-alone name of a month, 1 to 12 (ICU4X field set `M`).
    fn month(&self, month: u8, width: Width) -> String;

    /// Strategy: month and year as a heading, e.g. "November 2026" (ICU4X field set `YM`).
    fn month_title(&self, year: i32, month: u8, width: Width) -> String;

    /// Strategy: a day with its weekday as a heading, e.g. "Monday, 23 November" (ICU4X field
    /// set `MDE`).
    fn day_title(&self, year: i32, month: u8, day: u8, width: Width) -> String;

    /// Strategy: the first day of the week (CLDR week data of the region, or the person's choice
    /// by the BCP 47 extension `-u-fw`).
    fn first_weekday(&self) -> Weekday;
}
