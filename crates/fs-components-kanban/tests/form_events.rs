//! SW-2: the board over the write port: gestures become calls of `move` and `create`; `can` = false sends nothing; a refusal comes
//! back as its code.
//!
//! Mutants (each makes the named test fail): the neighbours are left out of the args -> `a_move_sends_the_column_id_and_the_neighbours`; `order` is sent again -> `a_move_never_writes_order`; `can`
//! is not asked (always sent) -> `nothing_is_sent_while_the_channel_may_not_be_used`; the column label is sent instead of the id ->
//! `a_move_sends_the_column_id_and_the_neighbours`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use std::sync::{Arc, Mutex};

use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_kanban::*;
use fs_components_kanban::{Card, Column};

struct Fake {
    allow: bool,
    sent: Mutex<Vec<String>>,
}

impl FormWrite for Fake {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        self.allow
    }
    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.sent
            .lock()
            .unwrap()
            .push(format!("{channel} {record:?} {args:?}"));
        if record == Some("bad") {
            return Err(WriteRefused::new("conflict"));
        }
        Ok(WriteDone::default())
    }
}

fn fake(allow: bool) -> Arc<Fake> {
    Arc::new(Fake {
        allow,
        sent: Mutex::default(),
    })
}

/// The target column `doing` holds a, b, c (and `c1` itself in `todo`); `empty` holds nothing.
fn board() -> Arc<FixedBoard> {
    let mut doing = Column::new("doing", "Doing");
    for k in ["a", "b", "c"] {
        doing.cards.push(Card::new(k, k));
    }
    let mut todo = Column::new("todo", "Todo");
    todo.cards.push(Card::new("c1", "c1"));
    Arc::new(FixedBoard {
        columns: vec![todo, doing, Column::new("empty", "Empty")],
        ops: vec![Operation::Move],
    })
}

/// What `move_card_at` sent as the args of the move, without the record.
fn sent_args(to: &str, index: Option<usize>) -> String {
    let f = fake(true);
    let ev = FormEvents::new(f.clone(), board());
    assert_eq!(ev.move_card_at("c1", "todo", to, index), Answer::Accepted);
    let sent = f.sent.lock().unwrap();
    sent[0].clone()
}

fn args(to: &str, pairs: &[(&str, &str)]) -> String {
    let mut s =
        format!("move Some(\"c1\") [WriteArg {{ name: \"column\", value: Text(\"{to}\") }}");
    for (n, v) in pairs {
        s += &format!(", WriteArg {{ name: \"{n}\", value: Text(\"{v}\") }}");
    }
    s + "]"
}

#[test]
fn a_move_sends_the_column_id_and_the_neighbours() {
    // between two neighbours: both
    assert_eq!(
        sent_args("doing", Some(1)),
        args("doing", &[("after", "a"), ("before", "b")])
    );
    // at the start: only `before`
    assert_eq!(
        sent_args("doing", Some(0)),
        args("doing", &[("before", "a")])
    );
    // at the end (index past the last card, or no place): only `after`
    assert_eq!(
        sent_args("doing", Some(3)),
        args("doing", &[("after", "c")])
    );
    assert_eq!(sent_args("doing", None), args("doing", &[("after", "c")]));
    // an empty column: neither
    assert_eq!(sent_args("empty", Some(0)), args("empty", &[]));
    assert_eq!(sent_args("empty", None), args("empty", &[]));
}

#[test]
fn a_move_never_writes_order() {
    for i in [None, Some(0), Some(1), Some(9)] {
        assert!(!sent_args("doing", i).contains("order"));
    }
}

#[test]
fn a_reorder_in_its_own_column_leaves_the_card_out_of_the_neighbours() {
    let mut todo = Column::new("todo", "Todo");
    for k in ["x", "c1", "y"] {
        todo.cards.push(Card::new(k, k));
    }
    let board = Arc::new(FixedBoard {
        columns: vec![todo],
        ops: vec![],
    });
    let f = fake(true);
    FormEvents::new(f.clone(), board).move_card_at("c1", "todo", "todo", Some(1));
    // without c1 the column is x, y: place 1 is between them
    assert_eq!(
        f.sent.lock().unwrap()[0],
        args("todo", &[("after", "x"), ("before", "y")])
    );
}

#[test]
fn nothing_is_sent_while_the_channel_may_not_be_used() {
    let f = fake(false);
    let ev = FormEvents::new(f.clone(), board());
    assert_eq!(
        ev.move_card_at("c1", "todo", "doing", Some(0)),
        Answer::Refused("locked".into())
    );
    assert_eq!(ev.add_card("todo", "x"), Answer::Refused("locked".into()));
    assert!(f.sent.lock().unwrap().is_empty());
    let src = FormSupports::new(
        Arc::new(FixedBoard {
            columns: vec![],
            ops: vec![Operation::Move],
        }),
        f,
    );
    assert!(
        !src.supports(Operation::Move),
        "the host's `can` decides, not the inner source"
    );
}

#[test]
fn a_refusal_comes_back_as_its_code_and_create_sends_title_and_column() {
    let f = fake(true);
    let ev = FormEvents::new(f.clone(), board());
    assert_eq!(
        ev.move_card_at("bad", "a", "b", None),
        Answer::Refused("conflict".into())
    );
    assert_eq!(ev.add_card("todo", "Probe"), Answer::Accepted);
    assert!(f.sent.lock().unwrap()[1].starts_with("create None"));
    let src = FormSupports::new(
        Arc::new(FixedBoard {
            columns: vec![],
            ops: vec![],
        }),
        f,
    );
    assert!(src.supports(Operation::Move) && src.supports(Operation::AddCard));
    assert!(
        !src.supports(Operation::Rename),
        "other operations stay with the inner source"
    );
}

/// Mutants: the lane is left out -> `a_new_card_carries_the_lane_of_its_column`; an empty lane is sent for a column without one ->
/// `a_column_without_a_lane_sends_no_lane`.
#[test]
fn a_new_card_carries_the_lane_of_its_column() {
    let f = fake(true);
    let mut todo = Column::new("todo", "Todo");
    todo.lane = Some("b1".into());
    let board = Arc::new(FixedBoard {
        columns: vec![todo, Column::new("free", "Free")],
        ops: vec![Operation::AddCard],
    });
    let ev = FormEvents::new(f.clone(), board);
    assert_eq!(ev.add_card("todo", "Probe"), Answer::Accepted);
    assert_eq!(ev.add_card("free", "Probe"), Answer::Accepted);
    let sent = f.sent.lock().unwrap();
    assert!(
        sent[0].contains("\"title\"")
            && sent[0].contains("\"column\"")
            && sent[0].contains("\"lane\"")
    );
    assert!(sent[0].contains("b1"));
    assert!(
        !sent[1].contains("lane"),
        "no lane, not even an empty one: {}",
        sent[1]
    );
}
