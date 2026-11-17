//! The foot of a column (the quick entry) and the short menu of a column head.
//!
//! Pattern: Facade methods of `BoardBench` (split from `bench.rs`).

use super::bench::BoardBench;
use super::model::{Answer, ColumnAction, NewCard};
use super::quick::{Quick, submit};
use fs_components_widgets::plan::Entry;
use std::sync::PoisonError;
impl BoardBench {
    /// The quick entry of `column` (one per column, kept, so what is typed survives a redraw).
    pub(super) fn entry(&self, column: &str) -> Entry {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(column.to_owned())
            .or_insert_with(|| Entry::new(&self.labels.quick_hint))
            .clone()
    }

    /// Open the quick entry at the foot of `column`.
    pub fn open_quick(&self, column: &str) {
        self.lock().set_quick(Some(column));
        self.refresh();
    }

    /// Close the quick entry.
    pub fn close_quick(&self) {
        self.lock().set_quick(None);
        self.refresh();
    }

    /// Enter was pressed in the quick entry of `column` with `text` (Shift: at the top). One line makes one card, several ask first.
    /// If the program refuses, the text is put back into the entry and the reason stands under it: nothing typed is lost.
    pub fn quick_submit(&self, column: &str, text: &str, top: bool) {
        match submit(column, text, top) {
            Quick::Nothing => {}
            Quick::Create(cards) => self.create(column, text, &cards),
            Quick::Confirm(cards) => {
                *self.many.lock().unwrap_or_else(PoisonError::into_inner) = Some(cards);
                self.entry(column).set(text);
                self.refresh();
            }
        }
    }

    pub(super) fn create(&self, column: &str, text: &str, cards: &[NewCard]) {
        let entry = self.entry(column);
        for card in cards {
            if let Answer::Refused(reason) = self.events.new_card(card) {
                entry.set(text);
                entry.set_notice(Some(&reason.text));
                self.refresh();
                return;
            }
        }
        entry.set_notice(None);
        self.lock().load(self.source.as_ref());
        self.refresh();
    }

    /// The reader answered "Create N cards?".
    pub fn answer_many(&self, yes: bool) {
        let cards = self
            .many
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let (true, Some(cards)) = (yes, cards) {
            if let Some(first) = cards.first() {
                let column = first.column.clone();
                let text = cards
                    .iter()
                    .map(|c| c.title.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.entry(&column).clear();
                self.create(&column, &text, &cards);
            }
        } else {
            self.refresh();
        }
    }

    /// The cards waiting for the reader's yes ("Create N cards?").
    pub fn pending_many(&self) -> Option<Vec<NewCard>> {
        self.many
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Something chosen in a column's short menu: the program does it (it owns names, limits and the table).
    pub fn column_action(&self, column: &str, action: ColumnAction) {
        self.events.column_action(column, action);
    }
}
