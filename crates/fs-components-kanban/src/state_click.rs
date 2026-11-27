//! The click-to-move: a card is held, then a click on a column puts it down there; and the rights check that every road shares.
//!
//! Pattern: State + Command (split from `state.rs`). Two small entrances into [`BoardState::place`] (the click on a column and the
//! withdrawn right); the rules stay in `place`, so a click, the mouse and a menu command give the same [`MoveRequest`]. No column
//! count is assumed: a column is named by its key, wherever the view put it (the wrap is the view's).

use crate::requests::Reason;
use crate::source::BoardSource;
use crate::state::{Action, BoardState};

impl BoardState {
    /// The right to move was taken back (the host's `supports(Move)` is false now): a held or carried card is given up where it
    /// was, nothing is sent, and the reason is said. `None` while the right stands.
    pub(crate) fn rights_gone(&mut self) -> Option<Action> {
        if self.can_move {
            return None;
        }
        if self.sorter.grabbed().is_some() {
            self.sorter.cancel();
            self.grab_origin = None;
        }
        Some(self.refuse(&Reason::new("locked", "Moving cards is not possible now.")))
    }

    /// A click on column `column` while a card is held: the card is put down at the end of it. Nothing held: nothing happens.
    /// The column the card already stands in just puts it down again. The held card is first given back to where it came from, so
    /// the request and its undo name the true origin.
    pub fn click_cell(&mut self, column: &str, source: &dyn BoardSource) -> Action {
        let Some(card) = self.sorter.grabbed().map(str::to_owned) else {
            return Action::None;
        };
        if let Some(gone) = self.rights_gone() {
            return gone;
        }
        self.sorter.cancel();
        self.grab_origin = None;
        let Some(lane) = self.sorter.lanes().iter().find(|l| l.key == column) else {
            return Action::Redraw;
        };
        if lane.items.iter().any(|i| i.key == card) {
            return Action::Redraw;
        }
        let end = lane.items.len();
        self.place(&card, column, end, source, false)
    }
}
