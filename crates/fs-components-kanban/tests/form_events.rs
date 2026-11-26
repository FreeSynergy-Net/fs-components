//! SW-2: the board over the write port: gestures become calls of `move` and `create`; `can` = false sends nothing; a refusal comes
//! back as its code (and a sentence). The adapters of the one board element (`BoardSource` / `BoardEvents`).
//!
//! Mutants (each makes the named test fail): the neighbours are left out of the args -> `a_move_sends_the_column_id_and_the_neighbours`; `order` is sent again -> `a_move_never_writes_order`; `can`
//! is not asked (always sent) -> `nothing_is_sent_while_the_channel_may_not_be_used`; the column label is sent instead of the id ->
//! `a_move_sends_the_column_id_and_the_neighbours`; the place is taken from `index` although `after` names a card ->
//! `the_card_it_comes_after_decides_the_place`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[macro_use]
mod common;

use std::sync::{Arc, Mutex};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_core::states::Load;
use fs_components_fields::KeysAsText;
use fs_components_kanban::{Answer, BoardEvents, BoardSource, MemoryBoard, MoveRequest, NewCard};
use fs_components_kanban::{Card, Column, FormEvents, FormSource, Operation, columns_of};

struct Fake {
    allow: bool,
    sent: Mutex<Vec<String>>,
}

impl FormWrite for Fake {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        self.allow
    }
    gestures_by_channel!();
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

fn events(f: &Arc<Fake>, board: MemoryBoard) -> FormEvents {
    FormEvents::new(f.clone(), Arc::new(board), Arc::new(KeysAsText))
}

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

/// The target column `doing` holds a, b, c (and `c1` itself in `todo`); `empty` holds nothing.
fn board() -> MemoryBoard {
    let mut doing = Column::new("doing", "Doing");
    for k in ["a", "b", "c"] {
        doing.cards.push(Card::new(k, k));
    }
    let mut todo = Column::new("todo", "Todo");
    todo.cards.push(Card::new("c1", "c1"));
    MemoryBoard::new(vec![todo, doing, Column::new("empty", "Empty")])
}

fn request(to: &str, index: usize, after: Option<&str>) -> MoveRequest {
    MoveRequest::new("c1", "todo", to, index, after.map(str::to_owned))
}

/// What `moved` sent as the args of the move, without the record.
fn sent_args(to: &str, index: usize, after: Option<&str>) -> String {
    let f = fake(true);
    let ev = events(&f, board());
    assert_eq!(ev.moved(&request(to, index, after)), Answer::Accepted);
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
        sent_args("doing", 1, Some("a")),
        args("doing", &[("after", "a"), ("before", "b")])
    );
    // at the start: only `before`
    assert_eq!(
        sent_args("doing", 0, None),
        args("doing", &[("before", "a")])
    );
    // at the end: only `after`
    assert_eq!(
        sent_args("doing", 3, Some("c")),
        args("doing", &[("after", "c")])
    );
    // an empty column: neither
    assert_eq!(sent_args("empty", 0, None), args("empty", &[]));
}

#[test]
fn the_card_it_comes_after_decides_the_place() {
    // a filter can hide cards: the number is of the shown ones, the card is what the module knows
    assert_eq!(
        sent_args("doing", 0, Some("b")),
        args("doing", &[("after", "b"), ("before", "c")])
    );
    // a card that is not there (gone meanwhile): the number decides, past the end counts as the end
    assert_eq!(
        sent_args("doing", 9, Some("zz")),
        args("doing", &[("after", "c")])
    );
    assert_eq!(
        sent_args("doing", 1, Some("zz")),
        args("doing", &[("after", "a"), ("before", "b")])
    );
}

#[test]
fn a_move_never_writes_order() {
    for (i, a) in [(0, None), (1, Some("a")), (9, Some("c"))] {
        assert!(!sent_args("doing", i, a).contains("order"));
    }
}

#[test]
fn a_reorder_in_its_own_column_leaves_the_card_out_of_the_neighbours() {
    let mut todo = Column::new("todo", "Todo");
    for k in ["x", "c1", "y"] {
        todo.cards.push(Card::new(k, k));
    }
    let f = fake(true);
    events(&f, MemoryBoard::new(vec![todo])).moved(&request("todo", 1, Some("x")));
    // without c1 the column is x, y: place 1 is between them
    assert_eq!(
        f.sent.lock().unwrap()[0],
        args("todo", &[("after", "x"), ("before", "y")])
    );
}

#[test]
fn nothing_is_sent_while_the_channel_may_not_be_used() {
    let f = fake(false);
    let ev = events(&f, board());
    let Answer::Refused(why) = ev.moved(&request("doing", 0, None)) else {
        panic!("a move without the channel must be refused");
    };
    assert_eq!(why.code, "locked");
    assert_eq!(why.said(), "kanban.refused.locked");
    assert!(matches!(
        ev.new_card(&NewCard::new("todo", "x", false)),
        Answer::Refused(r) if r.code == "locked"
    ));
    assert!(f.sent.lock().unwrap().is_empty());
    let src = FormSource::new(input(vec![]), f, Arc::new(KeysAsText));
    assert!(
        !src.supports(Operation::Move)
            && !src.supports(Operation::AddCard)
            && !src.columns_editable(),
        "the host's `can` decides"
    );
    assert!(src.locked("c1").is_some_and(|r| r.code == "locked"));
}

#[test]
fn a_refusal_comes_back_as_its_code_and_create_sends_title_and_column() {
    let f = fake(true);
    let ev = events(&f, board());
    let bad = MoveRequest::new("bad", "a", "b", 0, None);
    assert!(matches!(
        ev.moved(&bad),
        Answer::Refused(r) if r.code == "conflict" && r.said() == "kanban.refused.conflict"
    ));
    assert_eq!(
        ev.new_card(&NewCard::new("todo", "Probe", false)),
        Answer::Accepted
    );
    assert!(f.sent.lock().unwrap()[1].starts_with("create None"));
    let src = FormSource::new(input(vec![]), f, Arc::new(KeysAsText));
    assert!(
        src.supports(Operation::Move) && src.supports(Operation::AddCard) && src.columns_editable()
    );
    assert!(src.locked("c1").is_none());
    assert!(!src.offers_menu(), "the host has no menu to open");
}

/// The source follows the form: new rows count the generation up (the board reloads when it differs).
#[test]
fn new_rows_count_the_generation_up() {
    let row = |k: &str| FormRow::of(k, &[("title", k), ("column", "x")]);
    let src = FormSource::new(input(vec![row("a")]), fake(true), Arc::new(KeysAsText));
    let g = src.generation();
    assert_eq!(src.columns()[0].cards.len(), 1);
    src.set(input(vec![row("a"), row("b")]));
    assert_ne!(src.generation(), g);
    assert_eq!(src.columns()[0].cards.len(), 2);
}

/// Mutants: the lane is left out -> `a_new_card_carries_the_lane_of_its_column`; an empty lane is sent for a column without one ->
/// the same test (second half).
#[test]
fn a_new_card_carries_the_lane_of_its_column() {
    let f = fake(true);
    let mut todo = Column::new("todo", "Todo");
    todo.lane = Some("b1".into());
    let ev = events(
        &f,
        MemoryBoard::new(vec![todo, Column::new("free", "Free")]),
    );
    let new = |c: &str| ev.new_card(&NewCard::new(c, "Probe", false));
    assert_eq!(new("todo"), Answer::Accepted);
    assert_eq!(new("free"), Answer::Accepted);
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

/// Mutant: `columns_of` takes the lane of the first card -> this test fails (a column of two lanes sends no lane).
#[test]
fn a_column_of_two_lanes_sends_no_lane() {
    let row = |k: &str, l: &str| FormRow::of(k, &[("title", k), ("column", "x"), ("lane", l)]);
    let columns = columns_of(&input(vec![row("a", "b1"), row("b", "b2")]));
    let f = fake(true);
    let ev = events(&f, MemoryBoard::new(columns));
    assert_eq!(
        ev.new_card(&NewCard::new("x", "Probe", false)),
        Answer::Accepted
    );
    assert!(!f.sent.lock().unwrap()[0].contains("lane"));
}
