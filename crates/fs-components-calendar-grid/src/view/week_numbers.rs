//! The week numbers at the start of the week rows of the month grid.
//!
//! Pattern: Decorator — a column put before the month grid; the grid itself draws the same with or without it and knows
//! nothing of week numbers. Part `week-number` of the contract block `calendar-grid`.

use blinc_layout::div::{Div, div};

use super::parts::at;
use crate::handle::CalendarGrid;
use crate::iso_week::row_week;
use crate::layout::Rect;
use crate::state::GridState;

/// The width of the column: two digits of the caption size and the padding.
pub fn column_width(grid: &CalendarGrid) -> f32 {
    let s = grid.look.style;
    s.text.caption * 2.4 + s.spacing.xs * 2.0
}

/// The column: a short title in the head row, then the ISO week of each of the six rows (`head` and `row_h` are the
/// measures of the month grid next to it).
pub fn column(grid: &CalendarGrid, state: &GridState, width: f32, head: f32, row_h: f32) -> Div {
    let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
    let days = state.days();
    let mut col = div()
        .relative()
        .w(width)
        .h(head + row_h * 6.0)
        .id("cal-week-numbers");
    col = col.child(
        at(Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: head,
        })
        .padding_x_px(s.spacing.xs)
        .child(look.text(
            grid.say.say("calendar.week-short"),
            s.text.caption,
            pal.text_weak,
        )),
    );
    for (i, row) in days.chunks(7).enumerate() {
        let Some(week) = row_week(row) else { continue };
        col = col.child(
            at(Rect {
                x: 0.0,
                y: head + i as f32 * row_h,
                w: width,
                h: row_h,
            })
            .id(format!("cal-week-{i}"))
            .padding_x_px(s.spacing.xs)
            .padding_y_px(s.spacing.xs)
            .child(look.text(week.to_string(), s.text.caption, pal.text_weak)),
        );
    }
    col
}
