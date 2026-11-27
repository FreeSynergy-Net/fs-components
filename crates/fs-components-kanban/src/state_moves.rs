//! Moving a card: the one road every move takes, the program's answer, undo.
//!
//! Pattern: State + Command (the moves of `BoardState`; split from `state.rs`).

use crate::requests::{Answer, MoveRequest, Question, Reason};
use crate::source::BoardSource;
use crate::state::{Action, BoardState, Notice};
use fs_components_widgets::sortable_list::{Moved, Outcome, Place, Refusal};
impl BoardState {
    /// Put `card` at `index` of `column` (counted without the card itself) - the one road every move takes. A locked card or a
    /// column that does not take it gives a notice and no request; putting a card where it is gives nothing.
    pub fn place(
        &mut self,
        card: &str,
        column: &str,
        index: usize,
        source: &dyn BoardSource,
        confirmed: bool,
    ) -> Action {
        self.place_in(card, column, index, (None, None), source, confirmed)
    }

    /// [`place`](Self::place) with the lanes: `(from, to)`. A move to another lane of the SAME column is a request too (the column
    /// stays, the lane changes: the program changes the value the lanes are made from).
    pub fn place_in(
        &mut self,
        card: &str,
        column: &str,
        index: usize,
        lanes: (Option<String>, Option<String>),
        source: &dyn BoardSource,
        confirmed: bool,
    ) -> Action {
        if let Some(gone) = self.rights_gone() {
            return gone;
        }
        if let Some(reason) = self.reasons.get(card).cloned() {
            return self.refuse(&reason);
        }
        if let Err(reason) = source.can_drop(card, column, lanes.1.as_deref()) {
            return self.refuse(&reason);
        }
        let Some(before) = self.sorter.find(card) else {
            return Action::None;
        };
        let after_old = self.predecessor(card);
        let across = lanes.0 != lanes.1;
        match self.sorter.move_item(card, &Place::new(column, index)) {
            Outcome::Moved(m) => self.finish(&m, before.index, after_old, lanes, confirmed),
            Outcome::Unchanged if across => {
                let m = Moved::new(
                    card,
                    Place::new(column, before.index),
                    Place::new(column, index),
                );
                self.finish(&m, before.index, after_old, lanes, confirmed)
            }
            Outcome::Unchanged => Action::Redraw,
            Outcome::Refused(Refusal::LaneClosed) => self.refuse(&Reason::new(
                "closed",
                "This column does not take cards from elsewhere.",
            )),
            _ => Action::None,
        }
    }

    /// Where a card goes in `column` when it is dropped after the shown card `after` or before `before` (the neighbours of the place;
    /// both `None` = the end): the number the sorter counts (without the card itself).
    pub fn index_at(
        &self,
        column: &str,
        card: &str,
        after: Option<&str>,
        before: Option<&str>,
    ) -> usize {
        let rest: Vec<&str> = self
            .sorter
            .lanes()
            .iter()
            .find(|l| l.key == column)
            .map(|l| l.items.iter().map(|i| i.key.as_str()).collect())
            .unwrap_or_default();
        let rest: Vec<&str> = rest.into_iter().filter(|k| *k != card).collect();
        let at = |k: &str| rest.iter().position(|r| *r == k);
        after
            .and_then(at)
            .map(|p| p + 1)
            .or_else(|| before.and_then(at))
            .unwrap_or(rest.len())
    }

    /// The sorter has moved the card (optimistic): the request for the program and the request that takes it back.
    pub(crate) fn finish(
        &mut self,
        m: &Moved,
        old_index: usize,
        after_old: Option<String>,
        lanes: (Option<String>, Option<String>),
        confirmed: bool,
    ) -> Action {
        let after = self.predecessor(&m.item);
        let request = MoveRequest {
            card: m.item.clone(),
            from: m.from.lane.clone(),
            to: m.to.lane.clone(),
            from_lane: lanes.0.clone(),
            to_lane: lanes.1.clone(),
            index: m.to.index,
            after,
            confirmed,
        };
        let undo = MoveRequest {
            card: m.item.clone(),
            from: m.to.lane.clone(),
            to: m.from.lane.clone(),
            from_lane: lanes.1,
            to_lane: lanes.0,
            index: old_index,
            after: after_old,
            confirmed: true,
        };
        Action::Move(request, undo)
    }

    pub(crate) fn refuse(&mut self, reason: &Reason) -> Action {
        self.notice = Some(Notice {
            text: reason.said().to_owned(),
            undo: None,
            refused: true,
        });
        Action::Redraw
    }

    /// The program answered a move. `undo` is the request that takes it back (made by the workbench before the move).
    pub fn answered(
        &mut self,
        request: &MoveRequest,
        answer: Answer,
        undo: Option<MoveRequest>,
        moved_text: &str,
        source: &dyn BoardSource,
    ) {
        match answer {
            Answer::Accepted => {
                self.question = None;
                self.notice = Some(Notice {
                    text: moved_text.to_owned(),
                    undo,
                    refused: false,
                });
                self.load(source);
            }
            Answer::Refused(reason) => {
                self.question = None;
                self.notice = Some(Notice {
                    text: reason.said().to_owned(),
                    undo: None,
                    refused: true,
                });
                self.load(source);
            }
            Answer::Ask(question) => {
                self.question = Some((request.clone(), question));
                self.load(source);
            }
        }
    }

    /// The reader answered the program's question.
    pub fn take_question(&mut self) -> Option<(MoveRequest, Question)> {
        self.question.take()
    }

    /// The card before `card` in its column, as shown now (what `undo_of` needs, asked before a move).
    pub fn predecessor(&self, card: &str) -> Option<String> {
        let at = self.sorter.find(card)?;
        at.index
            .checked_sub(1)
            .and_then(|i| self.sorter.lanes()[at.lane].items.get(i))
            .map(|i| i.key.clone())
    }
}
