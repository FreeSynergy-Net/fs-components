//! The cells of the full month: day number, bars with titles over the days an item covers, "+N" where they do not fit.
//!
//! Pattern: Strategy (concrete) — the [`CellPainter`] of the grid without a variant. Bars that do not fit a cell are counted in a
//! "+N" mark instead of squeezed.

use blinc_layout::div::Div;
use fs_components_fields::Look;

use super::painter::{CellPainter, Frame};
use super::parts::{at, chip, day_number, item_color};
use crate::layout::Rect;
use crate::spans::span_rows;

/// Bars with titles.
pub struct Bars;

impl CellPainter for Bars {
    fn head(&self, f: &Frame, r: Rect, name: String) -> Div {
        let (look, s, pal) = (f.grid.look, f.grid.look.style, f.grid.look.palette());
        at(r)
            .padding_x_px(s.spacing.xs)
            .child(look.text(name, s.text.caption, pal.text_weak))
    }

    fn cell(&self, f: &Frame, i: usize) -> Div {
        let (look, s, pal) = (f.grid.look, f.grid.look.style, f.grid.look.palette());
        let day = f.days[i];
        let mut cell = f
            .base_cell(i)
            .border(1.0, Look::color(pal.border))
            .padding_x_px(s.spacing.xs)
            .padding_y_px(s.spacing.xs);
        if f.state.slot == Some(f.slot(i)) {
            cell = cell.bg(Look::color(pal.selected));
        }
        cell.child(day_number(
            &look,
            day.day(),
            f.state.today == Some(day),
            day.month() != f.state.anchor.month(),
        ))
    }

    fn over(&self, f: &Frame, mut root: Div) -> Div {
        let (grid, state, items) = (f.grid, f.state, f.items);
        let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
        let (cw, ch) = (f.cw, f.ch);
        let number_h = s.text.caption * 1.8 + s.spacing.xs;
        let bar_h = (s.text.caption * 1.6).ceil();
        let capacity = (((ch - number_h) / (bar_h + 2.0)).floor() as usize).max(1);
        for (row, bars) in span_rows(items, f.days, 7, false).iter().enumerate() {
            let top = f.head + row as f32 * ch + number_h;
            let mut hidden = [0usize; 7];
            for b in bars {
                if b.lane >= capacity {
                    (b.first..=b.last).for_each(|c| hidden[c] += 1);
                    continue;
                }
                let it = &items[b.index];
                let r = Rect {
                    x: b.first as f32 * cw + 2.0,
                    y: top + b.lane as f32 * (bar_h + 2.0),
                    w: (b.last - b.first + 1) as f32 * cw - 4.0,
                    h: bar_h,
                };
                let g = grid.clone();
                let key = it.key.clone();
                let selected = state.item.as_deref() == Some(it.key.as_str());
                root = root.child(
                    chip(&look, r, item_color(&look, it), &it.title, selected)
                        .id(format!("cal-item-{}-{row}", it.key))
                        .on_click(move |_| g.select_item(&key)),
                );
            }
            for (c, n) in hidden.iter().enumerate().filter(|(_, n)| **n > 0) {
                // The mark stands in the number row at the right of the cell: it never leaves the cell.
                let r = Rect {
                    x: c as f32 * cw + cw / 2.0,
                    y: top - number_h + s.spacing.xs,
                    w: cw / 2.0 - 4.0,
                    h: bar_h,
                };
                root = root.child(
                    at(r)
                        .id(format!("cal-more-{row}-{c}"))
                        .pointer_events_none()
                        .flex_row()
                        .justify_end()
                        .child(look.text(grid.say.more(*n), s.text.caption, pal.text_weak)),
                );
            }
        }
        root
    }
}
