//! The month grid: a head row, six rows of seven day cells, what the cells hold is the painter's.
//!
//! Pattern: View function — pixel positions come from `layout`; the look of the cells (bars with titles, or marks) is the
//! [`CellPainter`] of the variant. A click on a cell selects its day. With week numbers on, their column stands in front.

use blinc_layout::div::{Div, div};

use super::painter::{Frame, painter_for};
use super::week_numbers;
use crate::handle::CalendarGrid;
use crate::layout::{Rect, month_row_height};
use crate::model::GridItem;
use crate::names::weekday_short;
use crate::state::GridState;

/// The month grid filling `width` x `height`; with week numbers on, their column stands in front and the cells share the rest.
pub fn month(grid: &CalendarGrid, items: &[GridItem], width: f32, height: f32) -> Div {
    let state = grid.snapshot();
    if !state.week_numbers {
        return cells(grid, &state, items, width, height);
    }
    let s = grid.look.style;
    let head = s.heights.row_dense;
    let row_h = month_row_height(height, head, s.heights.row);
    let col_w = week_numbers::column_width(grid, &state);
    let numbers = week_numbers::column(grid, &state, col_w, head, row_h);
    div().flex_row().w(width).child(numbers).child(cells(
        grid,
        &state,
        items,
        (width - col_w).max(0.0),
        height,
    ))
}

fn cells(
    grid: &CalendarGrid,
    state: &GridState,
    items: &[GridItem],
    width: f32,
    height: f32,
) -> Div {
    let s = grid.look.style;
    let days = state.days();
    let head = s.heights.row_dense;
    let ch = month_row_height(height, head, s.heights.row);
    let f = Frame {
        grid,
        state,
        items,
        days: &days,
        cw: width / 7.0,
        ch,
        head,
    };
    let painter = painter_for(state);
    let mut root = div().relative().w(width).h(ch * 6.0 + head).id("cal-month");
    for col in 0..7 {
        let name = weekday_short(
            &*grid.dates,
            ((usize::from(state.week_start) + col) % 7) as u8,
        );
        let r = Rect {
            x: col as f32 * f.cw,
            y: 0.0,
            w: f.cw,
            h: head,
        };
        root = root.child(painter.head(&f, r, name));
    }
    for i in 0..days.len() {
        root = root.child(painter.cell(&f, i));
    }
    painter.over(&f, root)
}
