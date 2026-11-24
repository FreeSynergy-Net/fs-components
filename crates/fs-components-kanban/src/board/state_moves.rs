//! Moving a card: the one road every move takes, the program's answer, undo.
//!
//! Pattern: State + Command (the moves of `BoardState`; split from `state.rs`).

use super::model::{Answer, MoveRequest, Question, Reason};
use super::source::BoardSource;
use super::state::{Action, BoardState, Notice};
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
        if let Some(reason) = self.reasons.get(card).cloned() {
            return self.refuse(&reason);
        }
        if let Err(reason) = source.can_drop(card, column) {
            return self.refuse(&reason);
        }
        let Some(before) = self.sorter.find(card) else {
            return Action::None;
        };
        let after_old = self.predecessor(card);
        match self.sorter.move_item(card, &Place::new(column, index)) {
            Outcome::Moved(m) => self.finish(&m, before.index, after_old, confirmed),
            Outcome::Unchanged => Action::Redraw,
            Outcome::Refused(Refusal::LaneClosed) => self.refuse(&Reason::new(
                "closed",
                "This column does not take cards from elsewhere.",
            )),
            _ => Action::None,
        }
    }

    /// The sorter has moved the card (optimistic): the request for the program and the request that takes it back.
    pub(super) fn finish(
        &mut self,
        m: &Moved,
        old_index: usize,
        after_old: Option<String>,
        confirmed: bool,
    ) -> Action {
        let after = self.predecessor(&m.item);
        self.geometry = self.fresh_geometry();
        let request = MoveRequest {
            card: m.item.clone(),
            from: m.from.lane.clone(),
            to: m.to.lane.clone(),
            index: m.to.index,
            after,
            confirmed,
        };
        let undo = MoveRequest {
            card: m.item.clone(),
            from: m.to.lane.clone(),
            to: m.from.lane.clone(),
            index: old_index,
            after: after_old,
            confirmed: true,
        };
        Action::Move(request, undo)
    }

    pub(super) fn refuse(&mut self, reason: &Reason) -> Action {
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
