//! Which days a grid shows, and plain rectangles.
//!
//! Pattern: none — pure functions of days; the views take their positions from here.

use crate::Day;
use crate::model::Mode;

/// How many days the agenda shows.
pub const AGENDA_DAYS: u16 = 30;

/// A rectangle in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

/// The first day of the week holding `day`; `week_start` is 0 (Monday) .. 6 (Sunday).
pub fn week_of(day: Day, week_start: u8) -> Day {
    let back = (i64::from(day.weekday()) - i64::from(week_start % 7)).rem_euclid(7);
    day.add_days(-back)
}

/// The days a grid in `mode` shows around `anchor`: 42 (month), 7 (week) or 1.
pub fn shown_days(mode: Mode, anchor: Day, week_start: u8) -> Vec<Day> {
    let (first, n) = match mode {
        Mode::Month => (week_of(anchor.first_of_month(), week_start), 42),
        Mode::Week => (week_of(anchor, week_start), 7),
        Mode::Day => (anchor, 1),
        Mode::Agenda => (anchor, i64::from(AGENDA_DAYS)),
    };
    (0..n).map(|i| first.add_days(i)).collect()
}

/// The anchor one step on (back when `forward` is false).
pub fn step(mode: Mode, anchor: Day, forward: bool) -> Day {
    let sign: i32 = if forward { 1 } else { -1 };
    match mode {
        Mode::Month => anchor.add_months(sign),
        Mode::Week => anchor.add_days(7 * i64::from(sign)),
        Mode::Day => anchor.add_days(i64::from(sign)),
        Mode::Agenda => anchor.add_days(i64::from(sign) * i64::from(AGENDA_DAYS)),
    }
}

/// Rectangle of cell `index` in a grid of `cols` columns whose cells are `w` x `h`, below `top`.
#[allow(clippy::cast_precision_loss)] // a few dozen cells
pub fn cell_rect(index: usize, cols: usize, w: f32, h: f32, top: f32) -> Rect {
    Rect {
        x: (index % cols) as f32 * w,
        y: top + (index / cols) as f32 * h,
        w,
        h,
    }
}

/// The height of one of the six week rows of the month grid in `height` below a head row of `head`; never under `min_row`.
/// The cells and the week-number column take their rows from here, so they cannot drift apart.
pub fn month_row_height(height: f32, head: f32, min_row: f32) -> f32 {
    ((height - head) / 6.0).max(min_row)
}
