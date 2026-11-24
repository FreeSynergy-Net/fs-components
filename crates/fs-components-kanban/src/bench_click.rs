//! The click-to-move on the workbench, and the right to move asked live.
//!
//! Pattern: Facade methods of `BoardBench` (split from `bench.rs`) + Command (the same `place` as the mouse and the menu). The
//! right to move (`supports(Move)`) is asked of the program before every gesture, with the lock NOT held, so a host that reads
//! the board from inside `supports` cannot deadlock it, and a right taken back while a card is held is seen at once.

use crate::Operation;
use crate::bench::BoardBench;
use crate::state::Notice;

impl BoardBench {
    /// Ask the program whether it takes moves now and tell the state (never with the lock held).
    pub(crate) fn sync_rights(&self) {
        let live = self.source.supports(Operation::Move);
        self.lock().can_move = live;
    }

    /// The button went down on `card`. Without the right to move the card still opens on a click, and the board says "soon".
    pub(crate) fn press_card(&self, card: &str, window: (f32, f32)) {
        self.sync_rights();
        let movable = {
            let mut state = self.lock();
            state.press(card, window);
            state.can_move
        };
        if !movable {
            self.say_move_soon();
        }
    }

    /// The line under the board says moving is not supported yet (a locked control was pressed).
    pub(crate) fn say_move_soon(&self) {
        self.lock().notice = Some(Notice {
            text: self.labels.move_soon.clone(),
            undo: None,
            refused: false,
        });
        self.refresh();
    }

    /// A click on column `column` while a card is held puts the card down there (the click-to-move). Nothing held: nothing happens.
    pub fn click_cell(&self, column: &str) {
        self.sync_rights();
        let action = self.lock().click_cell(column, self.source.as_ref());
        self.perform(action);
    }

    /// A click on a card: with a card held it is a click on the card's column (put down there), otherwise it opens the card.
    pub(crate) fn card_click(&self, card: &str) {
        let column = {
            let state = self.lock();
            state.held().and_then(|_| {
                state
                    .lanes()
                    .iter()
                    .find(|l| l.items.iter().any(|i| i.key == card))
                    .map(|l| l.key.clone())
            })
        };
        match column {
            Some(column) => self.click_cell(&column),
            None => self.release(),
        }
    }
}
