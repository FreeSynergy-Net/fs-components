//! Days to pixels: the scale, rectangles of rows and items, the ticks of the axis.
//!
//! Pattern: none — pure functions; the view takes its positions from here and tests read them
//! without a window.

use crate::model::{Day, Item, Kind, Zoom};

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

/// Day 0 of the axis and pixels per day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    /// The first day on the axis.
    pub origin: Day,
    /// Pixels per day.
    pub px: f32,
}

impl Scale {
    /// Left edge of `day`.
    pub fn x_of(&self, day: Day) -> f32 {
        (day.number() - self.origin.number()) as f32 * self.px
    }

    /// The day at pixel `x`.
    pub fn day_at(&self, x: f32) -> Day {
        self.origin.add_days((x / self.px).floor() as i64)
    }
}

/// Heights of the rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rows {
    /// A row.
    pub row: f32,
    /// A bar inside it.
    pub bar: f32,
}

/// The first and last day to show: all items, `pad` days around; `fallback` when there are none.
pub fn range(items: &[Item], pad: i64, fallback: Day) -> (Day, Day) {
    let first = items.iter().map(|i| i.span().0).min().unwrap_or(fallback);
    let last = items.iter().map(|i| i.span().1).max().unwrap_or(fallback);
    (first.add_days(-pad), last.add_days(pad))
}

/// Where today is relative to the axis `first..=last`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodayMark {
    /// No today given.
    None,
    /// Today is on the axis: a line at this day.
    Line(Day),
    /// Today is before the axis: a mark at its left edge.
    Before,
    /// Today is after the axis: a mark at its right edge.
    After,
}

/// The mark for `today`. The axis never grows to reach today: it would cost time in proportion to
/// the distance (days to draw), so a today far away is an edge mark with an arrow.
pub fn today_mark(today: Option<Day>, first: Day, last: Day) -> TodayMark {
    match today {
        None => TodayMark::None,
        Some(d) if d < first => TodayMark::Before,
        Some(d) if d > last => TodayMark::After,
        Some(d) => TodayMark::Line(d),
    }
}

/// The rectangle of item number `row`. A milestone is a square centred on its day.
pub fn item_rect(item: &Item, row: usize, scale: &Scale, g: Rows) -> Rect {
    let y = row as f32 * g.row + (g.row - g.bar) / 2.0;
    match item.kind {
        Kind::Bar { start, end, .. } => Rect {
            x: scale.x_of(start),
            y,
            w: scale.x_of(end.add_days(1)) - scale.x_of(start),
            h: g.bar,
        },
        Kind::Milestone { day } => Rect {
            x: scale.x_of(day) + (scale.px - g.bar) / 2.0,
            y,
            w: g.bar,
            h: g.bar,
        },
    }
}

/// The part of a bar that is done.
pub fn progress_rect(bar: Rect, progress: u8) -> Rect {
    Rect {
        w: bar.w * f32::from(progress.min(100)) / 100.0,
        ..bar
    }
}

/// What a tick says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickLabel {
    /// The day of the month.
    Day(u8),
    /// A month (1..=12) and its year.
    Month(u8, i32),
}

/// A mark on the axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    /// Where it stands.
    pub day: Day,
    /// How many days it names (the width of its label).
    pub days: u16,
    /// What it says.
    pub label: TickLabel,
}

/// The lower row of the axis: days, weeks (Mondays) or months, by zoom.
pub fn ticks(zoom: Zoom, first: Day, last: Day) -> Vec<Tick> {
    let mut out = Vec::new();
    let mut d = first;
    while d <= last {
        let (y, m, n) = (d.year(), d.month(), d.day());
        let (label, days) = match zoom {
            Zoom::Day => (Some(TickLabel::Day(n)), 1),
            Zoom::Week => ((d.weekday() == 0).then_some(TickLabel::Day(n)), 7),
            Zoom::Month => (
                (n == 1 || d == first).then_some(TickLabel::Month(m, y)),
                i64::from(d.days_in_month() - n + 1),
            ),
        };
        if let Some(label) = label {
            out.push(Tick {
                day: d,
                days: u16::try_from(days).unwrap_or(1),
                label,
            });
        }
        d = d.add_days(1);
    }
    out
}

/// The upper row of the axis for day and week zoom: one tick per month shown.
pub fn month_marks(first: Day, last: Day) -> Vec<Tick> {
    ticks(Zoom::Month, first, last)
}

/// One row of the body: a lane heading or an item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BodyRow {
    /// The heading of a lane (shown where the lane begins).
    Lane(String),
    /// The item at this index of the source.
    Item(usize),
}

/// The rows of the body: a heading before each run of items of one lane (items without a lane get none), then the items in
/// order. Without lanes this is one row per item, as before.
pub fn body_rows(items: &[Item]) -> Vec<BodyRow> {
    let mut out = Vec::with_capacity(items.len());
    let mut last: Option<&str> = None;
    for (i, it) in items.iter().enumerate() {
        if let Some(l) = it.lane.as_deref()
            && last != Some(l)
        {
            out.push(BodyRow::Lane(l.to_string()));
        }
        last = it.lane.as_deref();
        out.push(BodyRow::Item(i));
    }
    out
}

/// The last day of the axis so that it reaches pixel `edge`: `last` itself when it already does, else the day that covers
/// `edge`. The axis (grid, ticks, today) ends where the view ends, not at the last item.
pub fn extend_to(last: Day, scale: &Scale, edge: f32) -> Day {
    if scale.x_of(last.add_days(1)) >= edge {
        last
    } else {
        scale.day_at(edge)
    }
}

/// Where the title of a bar stands when it does not fit in the bar: right of the bar, behind the first stretch of a dependency
/// arrow (`route::GAP`, the arrow leaves the bar there and turns down) and `pad`. `None` when `text_w` fits inside (with `pad`
/// on both sides).
pub fn outside_label_x(bar: Rect, text_w: f32, pad: f32) -> Option<f32> {
    (text_w + 2.0 * pad > bar.w).then_some(bar.x + bar.w + crate::route::GAP + pad)
}
