//! The cells of the small calendar: the number of the day and a mark under it when something is on that day.
//!
//! Pattern: Strategy (concrete) — the [`CellPainter`] of the variant `compact`; no bars and no titles, so the cells can be narrow.
//! Part `mark` of the contract block `calendar-grid`.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::painter::{CellPainter, Frame};
use super::parts::{at, day_number};
use crate::layout::Rect;

/// Number, and a mark under it for a day with items.
pub struct Marks;

impl CellPainter for Marks {
    fn head(&self, f: &Frame, r: Rect, name: String) -> Div {
        let (look, s, pal) = (f.grid.look, f.grid.look.style, f.grid.look.palette());
        at(r)
            .flex_row()
            .justify_center()
            .child(look.text(name, s.text.caption, pal.text_weak))
    }

    fn cell(&self, f: &Frame, i: usize) -> Div {
        let (look, s, pal) = (f.grid.look, f.grid.look.style, f.grid.look.palette());
        let day = f.days[i];
        let mut cell = f
            .base_cell(i)
            .flex_col()
            .items_center()
            .gap_px(s.spacing.xs / 2.0)
            .padding_y_px(s.spacing.xs)
            .rounded(s.radii.control);
        if f.state.slot == Some(f.slot(i)) {
            cell = cell.bg(Look::color(pal.selected));
        }
        cell = cell.child(day_number(
            &look,
            day.day(),
            f.state.today == Some(day),
            day.month() != f.state.anchor.month(),
        ));
        if f.has_items(i) {
            let size = (s.text.caption * 0.45).ceil();
            cell = cell.child(
                div()
                    .id(format!("cal-mark-{i}"))
                    .w(size)
                    .h(size)
                    .rounded(size / 2.0)
                    .bg(Look::color(pal.accent)),
            );
        }
        cell
    }

    fn over(&self, _f: &Frame, root: Div) -> Div {
        root
    }
}
