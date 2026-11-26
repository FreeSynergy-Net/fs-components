//! The plain data of the grid: items, ranges, slots, messages.
//!
//! Pattern: Value object — small immutable types the host fills; no program or JSCalendar type
//! appears here, times are already in the zone the reader sees.

use fs_components_styles::Rgba;

use crate::{Day, Moment};

/// Which grid is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Six weeks of day cells.
    Month,
    /// Seven day columns with a time axis.
    Week,
    /// One day column with a time axis.
    Day,
    /// The items of the next days as a list by day, days without items left out.
    Agenda,
}

impl Mode {
    /// All modes, in the order of the toolbar.
    pub const ALL: [Mode; 4] = [Mode::Month, Mode::Week, Mode::Day, Mode::Agenda];

    /// Name used in the ids of the words (`calendar-view-<name>`) and of the elements.
    pub fn name(self) -> &'static str {
        match self {
            Mode::Month => "month",
            Mode::Week => "week",
            Mode::Day => "day",
            Mode::Agenda => "agenda",
        }
    }
}

/// The role colour of an item without a colour of its own (the style decides the colour).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    /// The accent colour.
    Accent,
    /// Success colour.
    Success,
    /// Error colour.
    Error,
    /// No colour: the quiet look of the style (the default).
    #[default]
    Neutral,
}

/// One thing on the calendar. All-day items cover `start.day..=end.day`; timed items end at
/// `end` (exclusive), so one ending at midnight does not touch the next day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridItem {
    /// Identity, handed back by `select_item`.
    pub key: String,
    /// Title, already text.
    pub title: String,
    /// Start.
    pub start: Moment,
    /// End.
    pub end: Moment,
    /// Covers whole days.
    pub all_day: bool,
    /// The role colour when the item has no `colour`; `Neutral` (the default) is the quiet look of the style.
    pub tone: Tone,
    /// The content colour the module gave (`color`, RFC 8984 section 4.2.11), drawn as given; it wins over `tone`.
    pub colour: Option<Rgba>,
}

impl GridItem {
    /// A timed item.
    pub fn timed(key: &str, title: &str, start: Moment, end: Moment) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            start,
            end,
            all_day: false,
            tone: Tone::Neutral,
            colour: None,
        }
    }

    /// An all-day item over `first..=last`.
    pub fn all_day(key: &str, title: &str, first: Day, last: Day) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            start: Moment::start_of(first),
            end: Moment::start_of(last),
            all_day: true,
            tone: Tone::Neutral,
            colour: None,
        }
    }

    /// The same item in another role colour.
    #[must_use]
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// The same item in the content colour `colour`.
    #[must_use]
    pub fn colour(mut self, colour: Rgba) -> Self {
        self.colour = Some(colour);
        self
    }

    /// The last day the item touches.
    pub fn last_day(&self) -> Day {
        if self.all_day || self.end.minute > 0 || self.end.day == self.start.day {
            self.end.day.max(self.start.day)
        } else {
            self.end.day.add_days(-1)
        }
    }

    /// Lasts more than one day, or covers whole days: drawn as a bar, not in a time column.
    pub fn is_span(&self) -> bool {
        self.all_day || self.last_day() > self.start.day
    }
}

/// A place the reader chose: a day, and a minute when it was on a time axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The day.
    pub day: Day,
    /// Minute of the day; `None` for a day cell or the all-day row.
    pub minute: Option<u16>,
}

/// A navigation message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    /// One step back in the current mode.
    Previous,
    /// One step on.
    Next,
    /// Jump to today.
    Today,
}

/// How much of the grid is drawn: the look of the place (`variant` of the contract block).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Variant {
    /// The full grid: toolbar, bars with titles.
    #[default]
    Full,
    /// The small calendar of a side bar (the month only): no toolbar, a mark under a day with items instead of bars.
    Compact,
}

impl Variant {
    /// Name as in the contract (`compact`); `Full` is the block without a variant.
    pub fn name(self) -> &'static str {
        match self {
            Variant::Full => "full",
            Variant::Compact => "compact",
        }
    }
}
