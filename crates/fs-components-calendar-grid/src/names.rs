//! Calendar field names for the views: the bridge from the numbers of the grid to the CLDR names of `DateNames`.
//!
//! Pattern: Adapter — the grid counts weekdays 0 (Monday) to 6 (Sunday), `DateNames` counts ISO 1 to 7; nothing here holds a
//! word, every name comes from the injected [`DateNames`] (`DrawCtx.dates`).

use fs_components_words::{DateNames, Weekday, Width};

use crate::Day;

const WEEK: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

/// The weekday of the grid's number (0 Monday .. 6 Sunday; taken modulo seven).
pub fn weekday_of(n: u8) -> Weekday {
    WEEK[usize::from(n % 7)]
}

/// The grid's number of a weekday: 0 Monday .. 6 Sunday.
pub fn number_of(day: Weekday) -> u8 {
    day as u8 - 1
}

/// The first weekday the language and the person's setting want, as the grid counts it.
pub fn week_start(dates: &dyn DateNames) -> u8 {
    number_of(dates.first_weekday())
}

/// The short name of weekday `n` for a head (`Mo`, `Sun`: CLDR stand-alone, without the dot of the date patterns).
pub fn weekday_short(dates: &dyn DateNames, n: u8) -> String {
    dates.weekday(weekday_of(n), Width::Short)
}

/// The title of `day` in `width` (`Dienstag, 10. November`).
pub fn day_title(dates: &dyn DateNames, day: Day, width: Width) -> String {
    dates.day_title(day.year(), day.month(), day.day(), width)
}

/// The title of the month of `day` (`November 2026`).
pub fn month_title(dates: &dyn DateNames, day: Day) -> String {
    dates.month_title(day.year(), day.month(), Width::Long)
}
