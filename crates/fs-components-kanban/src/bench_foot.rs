//! The foot of a column (the quick entry) and the short menu of a column head.
//!
//! Pattern: Facade methods of `BoardBench` (split from `bench.rs`).

use crate::bench::BoardBench;
use crate::quick::{Quick, submit};
use crate::requests::{Answer, ColumnAction, NewCard};
use fs_components_widgets::plan::Entry;
use std::sync::PoisonError;
impl BoardBench {
    /// The quick entry of `column` (one per column, kept, so what is typed survives a redraw).
    pub(crate) fn entry(&self, column: &str) -> Entry {
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

    pub(crate) fn create(&self, column: &str, text: &str, cards: &[NewCard]) {
        let entry = self.entry(column);
        for card in cards {
            if let Answer::Refused(reason) = self.events.new_card(card) {
                entry.set(text);
                entry.set_notice(Some(reason.said()));
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

#[cfg(test)]
mod tests {
    //! Moved from the old quick-add tests (`view_quick_add`): what only this crate owns of the quick entry: a refusal gives the text
    //! back and says why, every column keeps its own text, an accepted card takes the notice away. (Trimming, blank lines and clearing
    //! the line are the `Entry` of `fs-components-widgets`, and `quick::submit`, tested in `board_model`.)
    //!
    //! Mutants: the text is not put back on a refusal -> `a_refusal_gives_the_text_back_and_says_why`; `create` clears all entries ->
    //! `a_card_in_one_column_leaves_the_text_of_another`.
    use std::sync::Arc;

    use crate::requests::{Answer, NewCard, Reason};
    use crate::{BoardBench, BoardEvents, Card, Column, Labels, MemoryBoard};

    struct Refuses;
    impl BoardEvents for Refuses {
        fn new_card(&self, _c: &NewCard) -> Answer {
            Answer::Refused(Reason::new("no-room", "No room here."))
        }
    }

    fn memory() -> MemoryBoard {
        let mut todo = Column::new("todo", "To do");
        todo.cards = vec![Card::new("a", "Write")];
        MemoryBoard::new(vec![todo, Column::new("done", "Done")])
    }

    #[test]
    fn a_refusal_gives_the_text_back_and_says_why() {
        let b = BoardBench::new(Arc::new(memory()), Labels::default()).events(Arc::new(Refuses));
        b.quick_submit("todo", "Pay the rent", false);
        assert_eq!(
            b.entry("todo").text(),
            "Pay the rent",
            "nothing typed is lost"
        );
        assert_eq!(b.entry("todo").notice().as_deref(), Some("No room here."));
    }

    #[test]
    fn a_card_in_one_column_leaves_the_text_of_another() {
        let m = Arc::new(memory());
        let b = BoardBench::new(m.clone(), Labels::default()).events(m.clone());
        b.entry("done").set("half a thought");
        b.entry("todo").set_notice(Some("old"));
        b.quick_submit("todo", "Pay the rent", false);
        assert_eq!(m.log(), ["new:todo:Pay the rent:false"]);
        assert_eq!(b.entry("done").text(), "half a thought");
        assert_eq!(
            b.entry("todo").notice(),
            None,
            "an accepted card takes the notice away"
        );
    }
}
