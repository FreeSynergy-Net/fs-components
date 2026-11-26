//! `NeutralDateNames`: the calendar names when there is no data for the language.
//!
//! Pattern: Null Object – answers every question of `DateNames` without language data, in the neutral form of the `mf2`
//! date functions (`Mon`, `11`, `2026-11`, week on Monday as in ISO 8601). There is no table of words of any language
//! here; `IcuDateNames` also falls back to these functions for a value that does not exist (month 13).

use fs_components_words::{DateNames, Weekday, Width};

/// The ISO 8601 abbreviations of the weekdays, the same for every width (a neutral text has no languages).
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// `Mon` … `Sun`.
pub(crate) fn weekday(day: Weekday) -> String {
    WEEKDAYS[usize::from(day as u8 - 1)].to_owned()
}

/// The month as a two-digit ISO number (`11`).
pub(crate) fn month(month: u8) -> String {
    format!("{month:02}")
}

/// Year and month in ISO 8601 form (`2026-11`).
pub(crate) fn month_title(year: i32, month: u8) -> String {
    format!("{year:04}-{month:02}")
}

/// The ISO date (`2026-11-26`), with the weekday in front when the date exists (`Thu 2026-11-26`).
pub(crate) fn day_title(year: i32, month: u8, day: u8) -> String {
    let iso = format!("{year:04}-{month:02}-{day:02}");
    match weekday_of(year, month, day) {
        Some(w) => format!("{} {iso}", self::weekday(w)),
        None => iso,
    }
}

/// The ISO weekday of a Gregorian date (Sakamoto's rule), `None` for a date that does not exist.
fn weekday_of(year: i32, month: u8, day: u8) -> Option<Weekday> {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let leap = |y: i32| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let feb = if leap(year) { 29 } else { 28 };
    let days = [31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&month) || day == 0 || i32::from(day) > days[usize::from(month) - 1] {
        return None;
    }
    let y = if month < 3 { year - 1 } else { year };
    let t = T[usize::from(month) - 1];
    let n = (y + y / 4 - y / 100 + y / 400 + t + i32::from(day)).rem_euclid(7);
    // n: 0 = Sunday … 6 = Saturday.
    Some(match n {
        1 => Weekday::Monday,
        2 => Weekday::Tuesday,
        3 => Weekday::Wednesday,
        4 => Weekday::Thursday,
        5 => Weekday::Friday,
        6 => Weekday::Saturday,
        _ => Weekday::Sunday,
    })
}

/// The date names without language data: ISO 8601 forms, the week starts on Monday.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeutralDateNames;

impl DateNames for NeutralDateNames {
    fn weekday(&self, day: Weekday, _width: Width) -> String {
        weekday(day)
    }
    fn month(&self, month: u8, _width: Width) -> String {
        self::month(month)
    }
    fn month_title(&self, year: i32, month: u8, _width: Width) -> String {
        self::month_title(year, month)
    }
    fn day_title(&self, year: i32, month: u8, day: u8, _width: Width) -> String {
        self::day_title(year, month, day)
    }
    fn first_weekday(&self) -> Weekday {
        Weekday::Monday
    }
}
