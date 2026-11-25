//! The form `bars-in-time`: its channels as plain data, and the source that hands them to the timeline.
//!
//! Pattern: Adapter — one record per item with the CHANNELS of the form (`items`, `start`, `end`, `label`, `lane`, `state`,
//! `depends`, `progress`; the host fills them through the map of fs-forms-run, as the kanban does for its form), turned into
//! [`Item`]s and [`Dependency`]s. No parser here: times arrive as [`Day`]s already (the host reads RFC 3339 and localises).
//! The timeline only SHOWS the plan; it changes nothing (the channel `move` is not read).
//!
//! Dependencies: a record that `depends` on another is drawn with an arrow from the end of the other to its own start (finish to
//! start, RFC 9253 `FINISHTOSTART`, the arrow of the crate). The other kinds (`STARTTOSTART` ...) are no channel of the form.

use std::collections::HashSet;

use fs_components_widgets::calendar_view::BarEntry;

use crate::model::{Day, Dependency, Item, Tone};
use crate::source::TimelineSource;

/// The form this view draws, in the spelling of the catalog (`forms` of the block `timeline`).
pub const FORM: &str = "bars-in-time@^1.0.0";

/// How a bar is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Tinted, the done part stronger.
    Tint,
    /// Outline only: nothing is going to happen (cancelled).
    Outline,
}

/// What a state code looks like: a colour slot of the theme (never a fixed colour) and a fill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateLook {
    /// Colour slot; the style turns it into its colour.
    pub tone: Tone,
    /// Fill: the pattern that tells the state without colour.
    pub fill: Fill,
}

/// The look of a state code. The codes are those of `progress` in RFC 8984 (JSCalendar `Task`, what the planning module
/// writes): `needs-action`, `in-process`, `completed`, `failed`, `cancelled`; any other code is neutral.
pub fn state_look(code: &str) -> StateLook {
    let (tone, fill) = match code {
        "in-process" => (Tone::Accent, Fill::Tint),
        "completed" => (Tone::Success, Fill::Tint),
        "failed" => (Tone::Error, Fill::Tint),
        "cancelled" => (Tone::Neutral, Fill::Outline),
        _ => (Tone::Neutral, Fill::Tint),
    };
    StateLook { tone, fill }
}

/// One record of the form: the channels, as the host maps them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarRecord {
    /// Identity of the record (the key `depends` of the others points at).
    pub key: String,
    /// Channel `label`: the name on the bar.
    pub label: String,
    /// Channel `start`, the first day.
    pub start: Day,
    /// Channel `end`, the last day (inclusive; an `end` before `start` counts as `start`).
    pub end: Day,
    /// Channel `lane`.
    pub lane: Option<String>,
    /// Channel `state`.
    pub state: Option<String>,
    /// Channel `depends`: keys of the records this one waits for.
    pub depends: Vec<String>,
    /// Channel `progress`, percent.
    pub progress: Option<u8>,
    /// A mark on one day (a record of no duration), not a bar.
    pub milestone: bool,
}

impl BarRecord {
    /// A record with the required channels (`label`, `start`, `end`) and nothing else.
    pub fn new(key: &str, label: &str, start: Day, end: Day) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            start,
            end,
            lane: None,
            state: None,
            depends: Vec::new(),
            progress: None,
            milestone: false,
        }
    }

    /// In a lane.
    #[must_use]
    pub fn lane(mut self, lane: &str) -> Self {
        self.lane = Some(lane.into());
        self
    }

    /// With a state code.
    #[must_use]
    pub fn state(mut self, state: &str) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Waiting for the record `key`.
    #[must_use]
    pub fn depends_on(mut self, key: &str) -> Self {
        self.depends.push(key.into());
        self
    }

    /// With a progress in percent.
    #[must_use]
    pub fn progress(mut self, percent: u8) -> Self {
        self.progress = Some(percent);
        self
    }

    /// The record of a row read by `read_bars` (the one reader of the form, shared with `calendar-grid` and `calendar-view`):
    /// `end` of the entry is exclusive, the record ends on its last day; a timed entry of no duration is a milestone.
    pub fn from_entry(entry: &BarEntry) -> Self {
        let last = if entry.end > entry.start && entry.end.minute == 0 {
            entry.end.day.add_days(-1)
        } else {
            entry.end.day
        };
        Self {
            key: entry.key.clone(),
            label: entry.label.clone(),
            start: entry.start.day,
            end: last,
            lane: entry.lane.clone(),
            state: entry.state.clone(),
            depends: entry.depends.clone(),
            progress: entry.progress,
            milestone: !entry.all_day && entry.start == entry.end,
        }
    }

    fn item(&self) -> Item {
        if self.milestone {
            let mut it = Item::milestone(&self.key, &self.label, self.start);
            it.lane.clone_from(&self.lane);
            return it;
        }
        // A completed task without a progress counts as done; any other without one as not started.
        let done = self
            .progress
            .unwrap_or(u8::from(self.state.as_deref() == Some("completed")) * 100);
        let mut it = Item::bar(&self.key, &self.label, self.start, self.end, done);
        it.lane.clone_from(&self.lane);
        if let Some(code) = &self.state {
            it.tone = state_look(code).tone;
            it.state = Some(code.clone());
        }
        it
    }
}

/// The records of a form as a source: bars grouped by lane (lanes in the order they first appear), arrows for `depends`.
#[derive(Clone, Debug, Default)]
pub struct BarsSource {
    records: Vec<BarRecord>,
}

impl BarsSource {
    /// A source over `records`.
    pub fn new(records: Vec<BarRecord>) -> Self {
        Self { records }
    }
}

impl TimelineSource for BarsSource {
    fn items(&self) -> Vec<Item> {
        let mut lanes: Vec<&Option<String>> = Vec::new();
        for r in &self.records {
            if !lanes.contains(&&r.lane) {
                lanes.push(&r.lane);
            }
        }
        lanes
            .into_iter()
            .flat_map(|l| self.records.iter().filter(move |r| &r.lane == l))
            .map(BarRecord::item)
            .collect()
    }

    fn dependencies(&self) -> Vec<Dependency> {
        let known: HashSet<&str> = self.records.iter().map(|r| r.key.as_str()).collect();
        self.records
            .iter()
            .flat_map(|r| {
                r.depends
                    .iter()
                    .filter(|d| known.contains(d.as_str()))
                    .map(|d| Dependency {
                        from: d.clone(),
                        to: r.key.clone(),
                    })
            })
            .collect()
    }
}
