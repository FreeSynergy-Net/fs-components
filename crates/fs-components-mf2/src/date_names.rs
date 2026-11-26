//! `IcuDateNames`: the CLDR names of calendar fields, read with ICU4X from the `names.blob` of a language package.
//!
//! Pattern: Adapter – the formatters of ICU4X (`E` weekday, `M` month, `YM` month title, `MDE` day title) and the CLDR week
//! data behind `fs_components_words::DateNames`. Each formatter is built once per (field set, length) and kept; the blob is
//! owned by the provider that built them (no `Box::leak`). A value that does not exist (month 13) gives the neutral text.

use fs_components_words::{DateNames, Weekday, Width};
use icu_calendar::week::{WeekInformation, WeekPreferences};
use icu_calendar::{Date, Gregorian};
use icu_datetime::options::Length;
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale::Locale as IcuLocale;
use icu_provider_blob::BlobDataProvider;

use crate::date_names_neutral as neutral;

/// One field set in its three lengths.
macro_rules! lengths {
    ($name:ident, $set:ident) => {
        struct $name {
            long: FixedCalendarDateTimeFormatter<Gregorian, fieldsets::$set>,
            medium: FixedCalendarDateTimeFormatter<Gregorian, fieldsets::$set>,
            short: FixedCalendarDateTimeFormatter<Gregorian, fieldsets::$set>,
        }
        impl $name {
            fn load(p: &BlobDataProvider, loc: &IcuLocale) -> Option<Self> {
                let one = |l: Length| {
                    FixedCalendarDateTimeFormatter::try_new_with_buffer_provider(
                        p,
                        loc.into(),
                        fieldsets::$set::for_length(l),
                    )
                    .ok()
                };
                Some(Self {
                    long: one(Length::Long)?,
                    medium: one(Length::Medium)?,
                    short: one(Length::Short)?,
                })
            }
            fn say(&self, width: Width, date: &Date<Gregorian>) -> String {
                let f = match width {
                    Width::Long => &self.long,
                    Width::Medium => &self.medium,
                    Width::Short => &self.short,
                };
                f.format(date).to_string()
            }
        }
    };
}
lengths!(Weekdays, E);
lengths!(Months, M);
lengths!(MonthTitles, YM);
lengths!(DayTitles, MDE);

/// The date names of one language and region from its package.
pub struct IcuDateNames {
    weekdays: Weekdays,
    months: Months,
    month_titles: MonthTitles,
    day_titles: DayTitles,
    first: Weekday,
}

/// A Monday: the weekday formatter only needs the weekday of the date it is given.
const MONDAY: (i32, u8, u8) = (2026, 11, 23);

impl IcuDateNames {
    /// The names for `tag`, or `None` when the blob has no data for this locale (the caller tries the next one).
    pub(crate) fn load(p: &BlobDataProvider, tag: &IcuLocale) -> Option<Self> {
        let week =
            WeekInformation::try_new_with_buffer_provider(p, WeekPreferences::from(tag)).ok()?;
        Some(Self {
            weekdays: Weekdays::load(p, tag)?,
            months: Months::load(p, tag)?,
            month_titles: MonthTitles::load(p, tag)?,
            day_titles: DayTitles::load(p, tag)?,
            first: from_icu(week.first_weekday),
        })
    }
}

fn from_icu(day: icu_calendar::types::Weekday) -> Weekday {
    use icu_calendar::types::Weekday as W;
    match day {
        W::Monday => Weekday::Monday,
        W::Tuesday => Weekday::Tuesday,
        W::Wednesday => Weekday::Wednesday,
        W::Thursday => Weekday::Thursday,
        W::Friday => Weekday::Friday,
        W::Saturday => Weekday::Saturday,
        W::Sunday => Weekday::Sunday,
    }
}

fn date(year: i32, month: u8, day: u8) -> Option<Date<Gregorian>> {
    Date::try_new_gregorian(year, month, day).ok()
}

impl DateNames for IcuDateNames {
    fn weekday(&self, day: Weekday, width: Width) -> String {
        let (y, m, d) = MONDAY;
        date(y, m, d + day as u8 - 1)
            .map_or_else(|| neutral::weekday(day), |dt| self.weekdays.say(width, &dt))
    }

    fn month(&self, month: u8, width: Width) -> String {
        date(2026, month, 1).map_or_else(|| neutral::month(month), |dt| self.months.say(width, &dt))
    }

    fn month_title(&self, year: i32, month: u8, width: Width) -> String {
        date(year, month, 1).map_or_else(
            || neutral::month_title(year, month),
            |dt| self.month_titles.say(width, &dt),
        )
    }

    fn day_title(&self, year: i32, month: u8, day: u8, width: Width) -> String {
        date(year, month, day).map_or_else(
            || neutral::day_title(year, month, day),
            |dt| self.day_titles.say(width, &dt),
        )
    }

    fn first_weekday(&self) -> Weekday {
        self.first
    }
}
