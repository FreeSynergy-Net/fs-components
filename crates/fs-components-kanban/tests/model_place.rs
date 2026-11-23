//! SW-2: the place a card is put down at (`MoveRequest.index`), reorder in a column, and the hand-over to the host
//! (`BoardEvents::move_card_at`, `can` = `supports(Move)`). No window.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use std::sync::{Arc, Mutex};

use fs_components_fields::{KeysAsText, Look};
use fs_components_kanban::*;
use fs_components_styles::StyleId;

fn cols() -> Vec<Column> {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![
        Card::new("a", "A"),
        Card::new("b", "B"),
        Card::new("c", "C"),
    ];
    let mut doing = Column::new("doing", "Doing");
    doing.cards = vec![Card::new("d", "D"), Card::new("e", "E")];
    vec![todo, doing, Column::new("done", "Done")]
}

fn mv(key: &str, from: &str, to: &str, index: Option<usize>) -> Step {
    Step::Move(MoveRequest {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        index,
    })
}

/// Between card 1 and card 2 of another column is place 1; after the last card (or on the column) is `None`.
#[test]
fn a_drop_between_two_cards_carries_the_place_and_the_end_carries_none() {
    let c = cols();
    let mut st = BoardState::default();
    st.grab(&c, true, "a");
    assert_eq!(st.hover_at(&c, "doing", Some(1)), Step::Changed);
    assert_eq!(st.drop_held(true), mv("a", "todo", "doing", Some(1)));

    st.grab(&c, true, "a");
    st.hover_at(&c, "doing", Some(0));
    assert_eq!(st.drop_held(true), mv("a", "todo", "doing", Some(0)));

    st.grab(&c, true, "a");
    st.hover_at(&c, "doing", Some(2)); // in front of a card that is not there: the end
    assert_eq!(st.drop_held(true), mv("a", "todo", "doing", None));

    st.grab(&c, true, "a");
    st.hover_at(&c, "doing", Some(1));
    st.hover(&c, "done"); // the column itself: the end, the old place is gone
    assert_eq!(st.drop_held(true), mv("a", "todo", "done", None));
}

/// In its own column a drop makes a `Move` (a reorder) with the place counted without the card; the old place is no move.
#[test]
fn a_drop_in_the_own_column_is_a_reorder() {
    let c = cols();
    let mut st = BoardState::default();
    // a (place 0) in front of card c (position 2): ends at place 1
    st.grab(&c, true, "a");
    st.hover_at(&c, "todo", Some(2));
    assert_eq!(st.drop_held(true), mv("a", "todo", "todo", Some(1)));
    // c to the front
    st.grab(&c, true, "c");
    st.hover_at(&c, "todo", Some(0));
    assert_eq!(st.drop_held(true), mv("c", "todo", "todo", Some(0)));
    // a to the end: place 2, never `None` in the own column
    st.grab(&c, true, "a");
    st.hover_at(&c, "todo", None);
    assert_eq!(st.drop_held(true), mv("a", "todo", "todo", Some(2)));
    // in front of itself or of the next card: it stays, nothing is asked of the host
    st.grab(&c, true, "b");
    st.hover_at(&c, "todo", Some(1));
    assert_eq!(st.drop_held(true), Step::Changed);
    st.grab(&c, true, "b");
    st.hover_at(&c, "todo", Some(2));
    assert_eq!(st.drop_held(true), Step::Changed);
}

/// Alt+Up / Alt+Down (mapped by the frontend) move one place in the column; at the edge nothing happens; locked refuses.
#[test]
fn reorder_by_key_moves_one_place() {
    let c = cols();
    let mut st = BoardState::default();
    assert_eq!(
        st.reorder(&c, true, "b", -1),
        mv("b", "todo", "todo", Some(0))
    );
    assert_eq!(
        st.reorder(&c, true, "b", 1),
        mv("b", "todo", "todo", Some(2))
    );
    assert_eq!(st.reorder(&c, true, "a", -1), Step::Ignored);
    assert_eq!(st.reorder(&c, true, "c", 1), Step::Ignored);
    assert_eq!(st.reorder(&c, true, "zz", 1), Step::Ignored);
    assert_eq!(st.reorder(&c, false, "b", 1), Step::Refused("locked"));
}

struct Host {
    ops: Vec<Operation>,
    log: Mutex<Vec<String>>,
}

impl BoardSource for Host {
    fn columns(&self) -> Vec<Column> {
        cols()
    }
    fn supports(&self, op: Operation) -> bool {
        self.ops.contains(&op)
    }
}

impl BoardEvents for Host {
    fn move_card_at(&self, key: &str, from: &str, to: &str, index: Option<usize>) -> Answer {
        self.log
            .lock()
            .unwrap()
            .push(format!("{key} {from}>{to} @{index:?}"));
        Answer::Accepted
    }
}

fn board(h: &Arc<Host>) -> KanbanBoard {
    KanbanBoard::new(
        h.clone(),
        Arc::new(KeysAsText),
        Look::new(StyleId::Apple, false),
    )
    .events(h.clone())
}

/// The host hears the place; with `Move` not supported it hears nothing (`can` = false: no send).
#[test]
fn the_host_hears_the_place_and_nothing_when_moving_is_locked() {
    let h = Arc::new(Host {
        ops: vec![Operation::Move],
        log: Mutex::default(),
    });
    let b = board(&h);
    b.grab("a");
    b.hover_at("doing", Some(1));
    b.drop_held();
    b.reorder("c", -1);
    assert_eq!(
        *h.log.lock().unwrap(),
        ["a todo>doing @Some(1)", "c todo>todo @Some(1)"]
    );

    let locked = Arc::new(Host {
        ops: vec![],
        log: Mutex::default(),
    });
    let b = board(&locked);
    assert_eq!(b.grab("a"), Step::Refused("locked"));
    assert_eq!(b.reorder("c", -1), Step::Refused("locked"));
    assert!(
        locked.log.lock().unwrap().is_empty(),
        "no send while locked"
    );
}

/// A host that knows no places keeps working: the default of `move_card_at` calls `move_card`.
#[test]
fn the_default_of_move_card_at_calls_move_card() {
    struct Old(Mutex<Vec<String>>);
    impl BoardEvents for Old {
        fn move_card(&self, key: &str, from: &str, to: &str) -> Answer {
            self.0.lock().unwrap().push(format!("{key} {from}>{to}"));
            Answer::Accepted
        }
    }
    let old = Old(Mutex::default());
    assert_eq!(
        old.move_card_at("a", "todo", "doing", Some(1)),
        Answer::Accepted
    );
    assert_eq!(*old.0.lock().unwrap(), ["a todo>doing"]);
}

/// The progress channel is a field of its own, not a fact: a card has it or not, and a card with a bar is taller.
#[test]
fn progress_is_its_own_optional_channel() {
    use fs_components_kanban::layout::card_height;
    let st = fs_components_styles::style(StyleId::Apple);
    let plain = Card::new("a", "A");
    let mut bar = Card::new("b", "B");
    bar.progress = Some(40);
    assert_eq!(plain.progress, None);
    assert!(plain.facts().is_empty() && bar.facts().is_empty());
    assert!(card_height(&bar, st) > card_height(&plain, st));
}
