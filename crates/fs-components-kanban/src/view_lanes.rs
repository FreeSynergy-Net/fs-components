//! The board in lanes: one row of column heads on top, under it one band per lane. A band has a head line (a disclosure: the arrow,
//! the name, the count, as "Default swimlane (7)" in Kanboard) and under it the columns with the cards of that lane.
//!
//! Pattern: Template method (the same head, card and column width as the plain board) + Adapter (`LanePlan` and the cards in, bands out).
//! Every lane the host named is drawn, empty or not; a lane that only the cards name (no head) is drawn too, at the end; cards that
//! stand in no lane are drawn in a last band "without lane". Nothing is hidden. The name is the text of the host (`LaneHead::label`),
//! never the key when a label is there. A card is not picked up by the pointer here (the hit geometry knows one row of columns).

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_widgets::disclosure::Disclosure;

use crate::bench::BoardBench;
use crate::hit::{CARD_GAP, CARD_H, COLUMN_GAP, COLUMN_W, HEAD_H, PAD};
use crate::state::BoardState;
use crate::swimlanes::{Lane, LaneCard, LaneHead, LanePlan, lanes_of};

/// Height of the head line of a band (the room the disclosure takes).
const BAND_HEAD_H: f32 = 56.0;
/// The key of the band of cards that stand in no lane.
const NONE_KEY: &str = "-none";

/// Which bands the reader closed, by lane key: it keeps over every redraw of the board (a band is drawn new each time, so its title
/// carries the count of now). Clones share it.
#[derive(Clone, Default)]
pub struct LaneMarks(Arc<Mutex<HashSet<String>>>);

impl LaneMarks {
    fn closed(&self) -> MutexGuard<'_, HashSet<String>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn is_open(&self, key: &str) -> bool {
        !self.closed().contains(key)
    }

    fn set(&self, key: &str, open: bool) {
        let mut closed = self.closed();
        if open {
            closed.remove(key);
        } else {
            closed.insert(key.to_owned());
        }
    }
}

/// The bands of a board: the lanes of `lanes_of` and, last, the lane of cards in no lane (its `headed` is `false`, key `NONE_KEY`).
pub(crate) fn bands(
    state: &BoardState,
    grouping: &str,
    heads: &[LaneHead],
    none_label: &str,
) -> Vec<Lane> {
    let cards: Vec<LaneCard> = state
        .lanes()
        .iter()
        .flat_map(|column| column.items.iter())
        .filter_map(|item| state.card(&item.key))
        .map(|card| LaneCard::of_card(card, grouping))
        .collect();
    let mut lanes = lanes_of(heads, &cards, grouping);
    let loose: Vec<String> = cards
        .iter()
        .filter(|c| lanes.iter().all(|l| !l.cards.contains(&c.key)))
        .map(|c| c.key.clone())
        .collect();
    if !loose.is_empty() {
        lanes.push(Lane {
            key: NONE_KEY.into(),
            label: none_label.into(),
            headed: false,
            cards: loose,
        });
    }
    lanes
}

impl BoardBench {
    /// The bands, when the host asked for lanes and sent their heads; `None` = the plain columns.
    pub(crate) fn lanes_of_state(&self, state: &BoardState) -> Option<Vec<Lane>> {
        match state.lane_plan() {
            LanePlan::On { grouping, heads } => {
                Some(bands(state, grouping, heads, &self.labels.lane_none))
            }
            _ => None,
        }
    }

    /// The size the bands need (width of the columns, height of the heads and of every band as it is open or closed).
    pub(crate) fn lanes_size(&self, state: &BoardState, lanes: &[Lane]) -> (f32, f32) {
        let (w, _) = state.geometry().size();
        let mut h = PAD * 2.0 + HEAD_H;
        for lane in lanes {
            h += BAND_HEAD_H;
            if self.lane_marks.is_open(&lane.key) {
                let tallest = state
                    .lanes()
                    .iter()
                    .map(|c| {
                        c.items
                            .iter()
                            .filter(|i| lane.cards.contains(&i.key))
                            .count()
                    })
                    .max()
                    .unwrap_or(0)
                    .max(1);
                h += tallest as f32 * (CARD_H + CARD_GAP) + PAD;
            }
        }
        (w, h)
    }

    /// The row of column heads and the bands under it.
    pub(crate) fn lanes_columns(&self, state: &BoardState, lanes: &[Lane]) -> Div {
        let mut heads = div()
            .class("fs-board-columns")
            .flex_row()
            .gap_px(COLUMN_GAP)
            .padding_x_px(PAD);
        for n in 0..state.heads().len() {
            heads = heads.child(self.head_row(state, n));
        }
        let mut all = div().class("fs-board-lanes").flex_col().child(heads);
        let shared = Arc::new(state.clone());
        for lane in lanes {
            all = all.child(self.band(&shared, lane));
        }
        all
    }

    fn band(&self, state: &Arc<BoardState>, lane: &Lane) -> Div {
        let id = format!("{}-lane-{}", self.id, lane.key);
        let title = format!("{} ({})", lane.label, lane.cards.len());
        let marks = self.lane_marks.clone();
        let key = lane.key.clone();
        let mut disclosure = Disclosure::new(&title)
            .id(&id)
            .on_toggle(move |open| marks.set(&key, open));
        if self.lane_marks.is_open(&lane.key) {
            disclosure = disclosure.opened();
        }
        let (this, state, lane_own) = (self.clone(), state.clone(), lane.clone());
        let mut band = div().id(format!("{id}-band")).class("fs-board-lane");
        if !lane.headed {
            band = band.class("fs-board-lane-unheaded");
        }
        band.padding_x_px(PAD)
            .child(disclosure.build(move || this.band_columns(&state, &lane_own)))
    }

    fn band_columns(&self, state: &BoardState, lane: &Lane) -> Div {
        let mut row = div()
            .class("fs-board-lane-columns")
            .flex_row()
            .gap_px(COLUMN_GAP);
        for (at, column) in state.lanes().iter().enumerate() {
            let mut cards = div()
                .class("fs-board-cards")
                .flex_col()
                .gap_px(CARD_GAP)
                .w(COLUMN_W)
                .attr("role", "list");
            let mut any = false;
            for item in column.items.iter().filter(|i| lane.cards.contains(&i.key)) {
                if let Some(card) = state.card(&item.key) {
                    cards = cards.child(self.card_in(state, card, Some(&lane.key)));
                    any = true;
                }
            }
            if !any && lane.cards.is_empty() && at == 0 {
                cards = cards.child(
                    text(self.labels.empty.clone())
                        .class("fs-board-empty")
                        .id(format!("{}-lane-{}-empty", self.id, lane.key)),
                );
            }
            row = row.child(cards.flex_shrink_0());
        }
        row
    }
}
