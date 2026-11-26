//! The cells of the small calendar: the number of the day and a mark under it when something is on that day.
//!
//! Pattern: Strategy (concrete) — the [`CellPainter`] of the variant `compact`; no bars and no titles, so the cells can be narrow.
//! Part `mark` of the contract block `calendar-grid`.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::painter::{CellPainter, Frame};
use super::parts::{at, day_number, item_look};
use crate::layout::Rect;

/// Most dots under a day (one per item, in the colour of the item).
const MAX_DOTS: usize = 3;

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
        let of_day: Vec<_> = f.items_of(i).take(MAX_DOTS).collect();
        if !of_day.is_empty() {
            let size = (s.text.caption * 0.45).ceil();
            let mut dots = div()
                .id(format!("cal-mark-{i}"))
                .flex_row()
                .gap_px(s.spacing.xs / 2.0);
            for it in of_day {
                let c = item_look(&look, it);
                let mut dot = div()
                    .w(size)
                    .h(size)
                    .rounded(size / 2.0)
                    .bg(Look::color(c.mark));
                if let Some(o) = c.outline {
                    dot = dot.border(1.0, Look::color(o));
                }
                dots = dots.child(dot);
            }
            cell = cell.child(dots);
        }
        cell
    }

    fn over(&self, _f: &Frame, root: Div) -> Div {
        root
    }
}
