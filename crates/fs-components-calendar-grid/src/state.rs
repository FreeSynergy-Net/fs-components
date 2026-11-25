//! The state of the grid: mode, anchor, today, selection, and what messages and keys do to it.
//!
//! Pattern: State — one struct with transitions; pure, no Blinc. Navigation messages
//! (`Nav`) and keys both end in the same transitions, so the buttons and the keyboard cannot disagree.

use fs_components_widgets::Key;

use crate::layout::{shown_days, step};
use crate::model::{Mode, Nav, Slot, Variant};
use crate::{Day, MINUTES_PER_DAY, Moment};

/// What a key press came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyOutcome {
    /// Not a key of the grid.
    Ignored,
    /// The cursor or the range moved.
    Moved,
    /// Enter or Space on the selected slot: the host should hear of it.
    Chosen(Slot),
}

/// Everything the grid remembers.
#[derive(Clone, Debug)]
pub struct GridState {
    /// Shown grid.
    pub mode: Mode,
    /// A day inside the shown range.
    pub anchor: Day,
    /// Today, marked; `None` marks nothing.
    pub today: Option<Day>,
    /// Now, drawn as a line on the time axis when its day is shown.
    pub now: Option<Moment>,
    /// First weekday, 0 Monday .. 6 Sunday.
    pub week_start: u8,
    /// Minutes per slot on the time axis.
    pub slot_minutes: u16,
    /// Selected slot.
    pub slot: Option<Slot>,
    /// Selected item key.
    pub item: Option<String>,
    /// Show the ISO week number in front of each week row of the month grid (`view.week-numbers`).
    pub week_numbers: bool,
    /// The look of the place (`compact` = the small calendar of a side bar).
    pub variant: Variant,
}

impl GridState {
    /// A grid in `mode` around `anchor`, Monday first, 30-minute slots, nothing selected.
    pub fn new(mode: Mode, anchor: Day) -> Self {
        Self {
            mode,
            anchor,
            today: None,
            now: None,
            week_start: 0,
            slot_minutes: 30,
            slot: None,
            item: None,
            week_numbers: false,
            variant: Variant::Full,
        }
    }

    /// The small calendar: the variant `compact` in the month (the other modes have no small form and draw as usual).
    pub fn is_mini(&self) -> bool {
        self.variant == Variant::Compact && self.mode == Mode::Month
    }

    /// The days shown.
    pub fn days(&self) -> Vec<Day> {
        shown_days(self.mode, self.anchor, self.week_start)
    }

    /// Apply a navigation message.
    pub fn navigate(&mut self, nav: Nav) {
        self.anchor = match nav {
            Nav::Previous => step(self.mode, self.anchor, false),
            Nav::Next => step(self.mode, self.anchor, true),
            Nav::Today => self.today.unwrap_or(self.anchor),
        };
    }

    /// Switch the grid; the anchor stays.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Select a slot (and drop an item selection: one thing is selected).
    pub fn select_slot(&mut self, slot: Slot) {
        self.slot = Some(slot);
        self.item = None;
    }

    /// Select an item.
    pub fn select_item(&mut self, key: &str) {
        self.item = Some(key.to_owned());
        self.slot = None;
    }

    /// A key: arrows move the selected slot, Page keys navigate, Home is today, Enter or Space choose.
    pub fn key(&mut self, key: Key) -> KeyOutcome {
        let timed = self.mode != Mode::Month;
        match key {
            Key::PageUp => self.navigate(Nav::Previous),
            Key::PageDown => self.navigate(Nav::Next),
            Key::Home => self.navigate(Nav::Today),
            Key::Left => self.shift_cursor(-1, 0),
            Key::Right => self.shift_cursor(1, 0),
            Key::Up => self.shift_cursor(if timed { 0 } else { -7 }, if timed { -1 } else { 0 }),
            Key::Down => self.shift_cursor(if timed { 0 } else { 7 }, i32::from(timed)),
            Key::Activate | Key::Enter | Key::Space => {
                return self.slot.map_or(KeyOutcome::Ignored, KeyOutcome::Chosen);
            }
            _ => return KeyOutcome::Ignored,
        }
        KeyOutcome::Moved
    }

    /// Move the cursor `days` days and `slots` slots; the anchor follows it.
    fn shift_cursor(&mut self, days: i64, slots: i32) {
        let from = self.slot.unwrap_or(Slot {
            day: self.anchor,
            minute: (self.mode != Mode::Month).then_some(8 * 60),
        });
        let mut day = from.day.add_days(days);
        let minute = from.minute.map(|m| {
            let moved = i32::from(m) + slots * i32::from(self.slot_minutes);
            let span = i32::from(MINUTES_PER_DAY);
            if moved < 0 {
                day = day.add_days(-1);
            } else if moved >= span {
                day = day.add_days(1);
            }
            u16::try_from(moved.rem_euclid(span)).unwrap_or(0)
        });
        self.slot = Some(Slot { day, minute });
        self.item = None;
        self.anchor = day;
    }
}
