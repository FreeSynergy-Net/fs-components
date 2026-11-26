//! What the cells of the month hold: the one place where the full grid and the small calendar differ.
//!
//! Pattern: Strategy (Zellmaler) — [`CellPainter`] draws a weekday head, a day cell and what lies over the cells; the month view
//! asks [`painter_for`] once and has no branch on the variant after that. Bars with titles are `Bars`, a mark under the day is `Marks`.

use blinc_layout::div::Div;

use super::parts::at;
use crate::Day;
use crate::handle::CalendarGrid;
use crate::layout::{Rect, cell_rect};
use crate::model::{GridItem, Slot};
use crate::state::GridState;

/// The measures and data of the month a painter draws.
pub struct Frame<'a> {
    /// The grid (look, words, click targets).
    pub grid: &'a CalendarGrid,
    /// The state now.
    pub state: &'a GridState,
    /// The items of the shown range.
    pub items: &'a [GridItem],
    /// The 42 days shown.
    pub days: &'a [Day],
    /// Width of a cell.
    pub cw: f32,
    /// Height of a week row.
    pub ch: f32,
    /// Height of the head row of weekday names.
    pub head: f32,
}

impl Frame<'_> {
    /// The rectangle of cell `i`.
    pub fn rect(&self, i: usize) -> Rect {
        cell_rect(i, 7, self.cw, self.ch, self.head)
    }

    /// The slot a click on cell `i` selects.
    pub fn slot(&self, i: usize) -> Slot {
        Slot {
            day: self.days[i],
            minute: None,
        }
    }

    /// The bare cell `i`: place, id and the click that selects its day (the painter adds its own look).
    pub fn base_cell(&self, i: usize) -> Div {
        let (g, slot) = (self.grid.clone(), self.slot(i));
        at(self.rect(i))
            .id(format!("cal-cell-{i}"))
            .on_click(move |_| g.select_slot(slot))
    }

    /// The items that touch cell `i`'s day.
    pub fn items_of(&self, i: usize) -> impl Iterator<Item = &GridItem> {
        let day = self.days[i];
        self.items
            .iter()
            .filter(move |it| it.start.day <= day && day <= it.last_day())
    }
}

/// The look of the cells of a month.
pub trait CellPainter {
    /// The head of weekday column `col`, showing `name`, in the box `r`.
    fn head(&self, f: &Frame, r: Rect, name: String) -> Div;
    /// Cell `i` with its day number (and what the painter shows inside the cell).
    fn cell(&self, f: &Frame, i: usize) -> Div;
    /// What lies over the cells (bars), added to `root`.
    fn over(&self, f: &Frame, root: Div) -> Div;
}

/// The painter of the variant of the state.
pub fn painter_for(state: &GridState) -> &'static dyn CellPainter {
    if state.is_mini() {
        &super::marks::Marks
    } else {
        &super::bars::Bars
    }
}
