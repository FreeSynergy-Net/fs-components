//! The pointer on the board: press, move, release.
//!
//! Pattern: State (the pointer part of `BoardState`; split from `state.rs`).

use crate::drag::{Drag, Moved as Pointer, Release};
use crate::source::BoardSource;
use crate::state::{Action, BoardState};
impl BoardState {
    /// The button went down on `card` at `window` (window coordinates). Without the right to move the press is still a click (the
    /// card opens), but the pointer never carries it.
    pub fn press(&mut self, card: &str, window: (f32, f32)) {
        let Some(from) = self
            .sorter
            .find(card)
            .and_then(|p| self.lane_key(p.lane))
            .map(str::to_owned)
        else {
            return;
        };
        self.drag.press(card.to_owned(), from, window);
    }

    /// The pointer moved: `window` for the 4 px rule, `local` in the columns row for the place.
    pub fn pointer(
        &mut self,
        window: (f32, f32),
        local: (f32, f32),
        source: &dyn BoardSource,
    ) -> Pointer {
        if !self.can_move {
            return Pointer::Nothing;
        }
        let moved = self.drag.pointer(window, local, &self.geometry);
        if let Drag::Dragging { card, over, .. } = &self.drag {
            // A locked card is not carried; the reason is said at once.
            if let Some(reason) = self.reasons.get(card).cloned() {
                self.blocked = Some(reason.clone());
                return moved;
            }
            self.blocked = over
                .and_then(|spot| self.lane_key(spot.column))
                .and_then(|column| source.can_drop(card, column).err());
        }
        moved
    }

    /// The button went up.
    pub fn release(&mut self, source: &dyn BoardSource) -> Action {
        self.blocked = None;
        match self.drag.release() {
            Release::Nothing | Release::Back => Action::Redraw,
            Release::Click(card) => Action::Open(card),
            Release::Drop { card, spot, .. } => {
                let Some(to) = self.lane_key(spot.column).map(str::to_owned) else {
                    return Action::Redraw;
                };
                self.place(&card, &to, spot.index, source, false)
            }
        }
    }
}
