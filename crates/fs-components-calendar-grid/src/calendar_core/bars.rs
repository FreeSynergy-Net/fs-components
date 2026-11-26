//! The form `bars-in-time` read as entries of the calendar components: the one place that maps its channels to the entries of
//! `calendar-grid` (agenda included) and `timeline`.
//!
//! Channels: `label`, `start`, `end` (RFC 3339 `date-time` or `full-date`), `all-day` (a flag, `true`/`false`), `lane`, `state`, `depends` (keys, several allowed),
//! `progress` (a number, cut to 0 to 100 and rounded; an unreadable one drops only the bar, never the row),
//! `color` (a CSS colour value read by `Rgba::from_css`; none or unreadable: no colour, the row stays, an unreadable one is counted).
//! The wall time of the offset the module gave is kept as it is: this adapter never converts between zones (localising is the
//! module's job; the rows come in the zone the view shows). A row that cannot be read is counted, never dropped silently.
//! The first day of the week is not part of this mapping.
//!
//! Pattern: Adapter (rows by channel in, entries out) as a pure function – every view of `bars-in-time` calls it, none keeps a copy.

use chrono::{DateTime, Datelike, Timelike};
use fs_components_core::form_read::{FormInput, FormRow, percent};
use fs_components_styles::Rgba;

use super::day::{Day, Moment};

/// One row of `bars-in-time`, read. `end` is exclusive; for an all-day entry it is the start of the day after the last one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarEntry {
    /// The key of the record.
    pub key: String,
    /// What it is called (the key when the row has no label).
    pub label: String,
    /// Where it begins (wall time).
    pub start: Moment,
    /// Where it ends, exclusive; never before `start`.
    pub end: Moment,
    /// Without a time of day.
    pub all_day: bool,
    /// The lane it belongs to, if the module says so.
    pub lane: Option<String>,
    /// The mark of the row (a level or state), if any.
    pub state: Option<String>,
    /// Keys of the rows this one waits for (channel `depends`, finish to start).
    pub depends: Vec<String>,
    /// Percent done (channel `progress`, 0 to 100), if the module says so.
    pub progress: Option<u8>,
    /// The colour of the channel `color` as given, if the module gave a readable one.
    pub color: Option<Rgba>,
}

/// What reading the rows gave: the entries in the order of the rows and the keys of the rows that could not be read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BarsRead {
    /// The readable rows.
    pub entries: Vec<BarEntry>,
    /// Keys of the rows left out because a value is missing or not a date, a time or a flag (the names in the notice of a partial list).
    pub invalid: Vec<String>,
    /// Keys of the rows whose `color` is not a colour: they stay in the list without a colour (a counted case, never a silent wrong one).
    pub invalid_color: Vec<String>,
}

/// A time as the module wrote it.
enum Stamp {
    /// `full-date`.
    Date(Day),
    /// `date-time`, as wall time.
    Time(Moment),
}

impl Stamp {
    fn day(&self) -> Day {
        match self {
            Self::Date(day) => *day,
            Self::Time(moment) => moment.day,
        }
    }

    fn moment(&self) -> Moment {
        match self {
            Self::Date(day) => Moment::start_of(*day),
            Self::Time(moment) => *moment,
        }
    }
}

fn stamp(text: &str) -> Option<Stamp> {
    if let Some(day) = Day::parse(text) {
        return Some(Stamp::Date(day));
    }
    let wall = DateTime::parse_from_rfc3339(text).ok()?.naive_local();
    let day = Day::new(
        wall.year(),
        u8::try_from(wall.month()).ok()?,
        u8::try_from(wall.day()).ok()?,
    )?;
    Some(Stamp::Time(Moment::at(
        day,
        u16::try_from(wall.hour()).ok()?,
        u16::try_from(wall.minute()).ok()?,
    )))
}

fn flag(row: &FormRow) -> Option<bool> {
    match row.get("all-day") {
        None | Some("false") => Some(false),
        Some("true") => Some(true),
        Some(_) => None,
    }
}

fn entry(row: &FormRow) -> Option<BarEntry> {
    let start = stamp(row.get("start")?)?;
    let end = match row.get("end") {
        Some(text) => stamp(text)?,
        None => match &start {
            Stamp::Date(day) => Stamp::Date(*day),
            Stamp::Time(moment) => Stamp::Time(*moment),
        },
    };
    let all_day = flag(row)? || matches!(start, Stamp::Date(_));
    let (start, end) = if all_day {
        let last = end.day();
        if last < start.day() {
            return None;
        }
        (
            Moment::start_of(start.day()),
            Moment::start_of(last.add_days(1)),
        )
    } else {
        if matches!(end, Stamp::Date(_)) || end.moment() < start.moment() {
            return None;
        }
        (start.moment(), end.moment())
    };
    // An unreadable `progress` drops only the bar, never the row.
    let progress = row.get("progress").and_then(percent);
    Some(BarEntry {
        key: row.key.clone(),
        label: row.get("label").unwrap_or(&row.key).to_owned(),
        start,
        end,
        all_day,
        lane: row.get("lane").map(str::to_owned),
        state: row.get("state").map(str::to_owned),
        depends: row.all("depends").into_iter().map(str::to_owned).collect(),
        progress,
        color: row.get("color").and_then(Rgba::from_css),
    })
}

/// The rows of `input` as entries; the keys of rows that cannot be read are collected in [`BarsRead::invalid`].
#[must_use]
pub fn read_bars(input: &FormInput) -> BarsRead {
    let mut read = BarsRead::default();
    for row in &input.rows {
        match entry(row) {
            Some(e) => {
                if row.get("color").is_some() && e.color.is_none() {
                    read.invalid_color.push(e.key.clone());
                }
                read.entries.push(e);
            }
            None => read.invalid.push(row.key.clone()),
        }
    }
    read
}
