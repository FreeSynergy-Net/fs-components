//! The ISO 8601 week number of a day.
//!
//! Pattern: Adapter — `chrono::NaiveDate::iso_week` (the standard, week 1 holds the first Thursday of the year) behind our
//! own `Day`, so no view knows chrono and nothing is calculated by hand.

use chrono::{Datelike, NaiveDate};

use crate::Day;

/// The ISO 8601 week (1..=53) of `day`; `None` when the day is no date of the calendar chrono knows.
pub fn iso_week(day: Day) -> Option<u8> {
    let date = NaiveDate::from_ymd_opt(day.year(), u32::from(day.month()), u32::from(day.day()))?;
    u8::try_from(date.iso_week().week()).ok()
}

/// The week number a row of days stands for: the ISO week of its Thursday, which is the day that decides the week whatever
/// the first weekday of the row is. A row without a Thursday (fewer than four days) has none.
pub fn row_week(row: &[Day]) -> Option<u8> {
    row.iter()
        .find(|d| d.weekday() == 3)
        .copied()
        .and_then(iso_week)
}
