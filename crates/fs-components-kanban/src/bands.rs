//! The lanes of a board in the form the view needs: the lanes of `lanes_of` and, last, the lane of cards that stand in no lane.
//!
//! Pattern: Adapter (the state and the plan of the host in, `Lane`s out; the pure rule is `swimlanes::lanes_of`).
//! Every lane the host named is drawn, empty or not; a lane that only the cards name (no head) is drawn too, at the end; cards that
//! stand in no lane are drawn in a last lane "without lane". Nothing is hidden. The name is the text of the host (`LaneHead::label`).

use crate::bench::BoardBench;
use crate::state::BoardState;
use crate::swimlanes::{Lane, LaneCard, LaneHead, LanePlan, lanes_of};

/// The key of the lane of cards that stand in no lane (a drop there says `to_lane` = this key; the form adapter sends an empty `lane`: the value is taken away).
pub const NONE_KEY: &str = "-none";

/// The lanes of a board: the lanes of `lanes_of` and, last, the lane of cards in no lane (its `headed` is `false`, key [`NONE_KEY`]).
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
    /// The lanes, when the host asked for them and sent their heads; `None` = the plain columns.
    pub(crate) fn lanes_of_state(&self, state: &BoardState) -> Option<Vec<Lane>> {
        match state.lane_plan() {
            LanePlan::On { grouping, heads } => {
                Some(bands(state, grouping, heads, &self.labels.lane_none))
            }
            _ => None,
        }
    }
}
