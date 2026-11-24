//! The context menu of the board and the commands `board.*`.
//!
//! Pattern: Command (split from `bench.rs`): the workbench says which lines a card has and runs the id it gets back.

use super::bench::BoardBench;
use super::commands;
use super::model::{ColumnAction, ContextTarget};
use std::sync::Arc;
use std::sync::atomic::Ordering;
impl BoardBench {
    /// The reader asked for a context menu at `at` (window coordinates): the board's lines first, then the program's.
    pub fn context(&self, target: &ContextTarget, at: (f32, f32)) {
        self.ask_context(target, at, true);
    }

    /// Like [`BoardBench::context`] for a press on a button (the column's "..." button): no right click is on its way to the board.
    pub fn context_by_click(&self, target: &ContextTarget, at: (f32, f32)) {
        self.ask_context(target, at, false);
    }

    pub(super) fn ask_context(&self, target: &ContextTarget, at: (f32, f32), right_click: bool) {
        let supports = self.source.supports();
        if !supports.context {
            // No menu in the host: the "..." button of a column hands the column to the program (it asks what to do with it).
            if let (ContextTarget::Column(column), false, true) =
                (target, right_click, supports.columns_edit)
            {
                self.column_action(column, ColumnAction::EditColumns);
            }
            return;
        }
        // A right click bubbles from the card to the board: the inner one claims it, the board's own handler then stands aside.
        match target {
            ContextTarget::Board if self.claimed.swap(false, Ordering::Relaxed) => return,
            ContextTarget::Board => {}
            _ if right_click => self.claimed.store(true, Ordering::Relaxed),
            _ => {}
        }
        let ids = match target {
            ContextTarget::Card(card) => self.card_menu(card, at),
            ContextTarget::Column(column) => {
                let mut ids = vec![commands::NEW.to_owned()];
                ids.extend(self.events.context(target, at));
                let _ = column;
                ids
            }
            ContextTarget::Board => self.events.context(target, at),
        };
        if let Some(hook) = &self.hook {
            hook(target, at, &ids);
        }
    }

    /// Run a command id of the board (from a context menu line or a shortcut) on `card`: `board.open`, `board.up`, `board.down`,
    /// `board.top`, `board.bottom`, `board.to:<column>`, `board.undo`. False for an id that is not the board's.
    pub fn run(&self, id: &str, card: &str) -> bool {
        if id == commands::UNDO {
            self.undo();
            return true;
        }
        if id == commands::OPEN {
            self.open(card);
            return true;
        }
        let action = {
            let mut state = self.lock();
            let Some(at) = state.lanes().iter().enumerate().find_map(|(l, lane)| {
                lane.items
                    .iter()
                    .position(|i| i.key == card)
                    .map(|i| (l, i, lane.items.len()))
            }) else {
                return false;
            };
            let column = state.lanes()[at.0].key.clone();
            let source = Arc::clone(&self.source);
            match id {
                commands::UP => state.place(
                    card,
                    &column,
                    at.1.saturating_sub(1),
                    source.as_ref(),
                    false,
                ),
                commands::DOWN => state.place(
                    card,
                    &column,
                    (at.1 + 1).min(at.2 - 1),
                    source.as_ref(),
                    false,
                ),
                commands::TOP => state.place(card, &column, 0, source.as_ref(), false),
                commands::BOTTOM => state.place(card, &column, at.2 - 1, source.as_ref(), false),
                other => match commands::column_of(other) {
                    Some(to) => {
                        let end = state
                            .lanes()
                            .iter()
                            .find(|l| l.key == to)
                            .map_or(0, |l| l.items.len());
                        state.place(card, to, end, source.as_ref(), false)
                    }
                    None => return false,
                },
            }
        };
        self.perform(action);
        true
    }

    /// The lines of the menu of `card` (ids of the command model), as far as the program adds its own: the board's own come first.
    pub fn card_menu(&self, card: &str, at: (f32, f32)) -> Vec<String> {
        let target = ContextTarget::Card(card.to_owned());
        let own = {
            let state = self.lock();
            let Some((lane, index, len)) =
                state.lanes().iter().enumerate().find_map(|(l, lane)| {
                    lane.items
                        .iter()
                        .position(|i| i.key == card)
                        .map(|i| (l, i, lane.items.len()))
                })
            else {
                return Vec::new();
            };
            let others: Vec<String> = state
                .lanes()
                .iter()
                .enumerate()
                .filter(|(l, _)| *l != lane)
                .map(|(_, l)| l.key.clone())
                .collect();
            commands::card_lines(index, len, &others)
        };
        let mut ids: Vec<String> = own
            .into_iter()
            .filter(|l| l.enabled)
            .map(|l| l.id)
            .collect();
        ids.extend(self.events.context(&target, at));
        ids
    }
}
