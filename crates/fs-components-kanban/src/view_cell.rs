//! One cell of the board: the cards of a column (in a lane: of a column and a lane) and, in a plain board, the foot of the column.
//!
//! Pattern: Template method (the plain board and the lanes draw the same cell; only the ids and the foot differ).
//! The cell is the place a card can be put down (`<id>-col-<column>`, in a lane `<id>-lane-<lane>-col-<column>`): it is at least
//! `CELL_MIN_H` high, so an empty cell can be hit too.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::BoardState;
use crate::bench::BoardBench;
use crate::sizes::{CARD_GAP, CELL_MIN_H, COLUMN_W, PAD};

/// The indent of a level of the treegrid (16 px) in the units of `Div::ml` (4 px), as a step back.
const INDENT_UNITS: f32 = -4.0;
use crate::swimlanes::Lane;

impl BoardBench {
    /// The cell of column `n` (of `lane`, if the board has lanes).
    pub(crate) fn cell(&self, state: &BoardState, lane: Option<&Lane>, n: usize) -> Div {
        let id = &self.id;
        let head = &state.heads()[n];
        let key = &head.key;
        let (cell_id, cards_id, empty_id) = match lane {
            None => (
                format!("{id}-col-{key}"),
                format!("{id}-cards-{key}"),
                format!("{id}-empty-{key}"),
            ),
            Some(l) => (
                format!("{id}-lane-{}-col-{key}", l.key),
                format!("{id}-lane-{}-cards-{key}", l.key),
                format!("{id}-lane-{}-empty", l.key),
            ),
        };
        // The click-to-move targets the list of cards, not the foot or the "..." button: a click on a locked control must not put a
        // held card down.
        let (put_down, column_key) = (self.clone(), key.clone());
        let mut cards = div()
            .id(cards_id)
            .class("fs-board-cards")
            .flex_col()
            .gap_px(CARD_GAP)
            .w(COLUMN_W)
            .attr("role", "list")
            .on_click(move |_| put_down.click_cell(&column_key));
        let mut any = false;
        for item in state.lanes()[n]
            .items
            .iter()
            .filter(|i| lane.is_none_or(|l| l.cards.contains(&i.key)))
        {
            if let Some(card) = state.card(&item.key) {
                cards = cards.child(self.card_in(state, card, lane.map(|l| l.key.as_str())));
                any = true;
            }
        }
        // A plain column says it is empty; a lane says it once, in its first column, when it has no card at all.
        let says = match lane {
            None => !any,
            Some(l) => !any && l.cards.is_empty() && n == 0,
        };
        if says {
            cards = cards.child(
                text(self.labels.empty.clone())
                    .class("fs-board-empty")
                    .id(empty_id),
            );
        }
        let mut cell = div()
            .id(cell_id)
            .class("fs-board-cell")
            .flex_col()
            .w(COLUMN_W + 2.0 * PAD)
            .min_h(CELL_MIN_H)
            .padding_x_px(PAD)
            .padding_y_px(PAD)
            .attr("role", "group")
            .child(cards);
        if lane.is_none() {
            cell = cell.child(self.foot(state, key));
        } else if n == 0 {
            // The first cell of a treegrid row stands behind the indent of its level: the cell takes it back so the columns line up.
            cell = cell.ml(INDENT_UNITS);
        }
        cell
    }
}
