//! G8a-r2: WHICH gesture the board fires, checked on the gesture itself (trigger, entry, record), not on a channel name made from it:
//! `moved` fires `drop` for the card, `new_card` fires `quick-add` for no card, never an entry (only `menu` has entries). Without a
//! `may_fire` of the host (`NoWrite`) the board is locked with the sentence `kanban.refused.gesture-unbound`.
//!
//! Mutants (each makes the named test fail): `moved` fires `activate` -> `a_move_fires_the_gesture_drop_for_the_card`; `new_card`
//! fires `menu` or an entry -> `a_new_card_fires_the_gesture_quick_add_without_a_record`; the gesture of `locked` / `supports` is
//! swapped -> `both_gestures_are_asked_before_they_are_fired_and_by_the_same_words`; the refusal code is made by the adapter ->
//! `a_host_without_gestures_locks_the_board_with_its_own_sentence`.
//!
//! Pattern: none (tests; the double records, it is the Spy of the port `FormWrite`).
#![allow(clippy::pedantic)]

use std::sync::{Arc, Mutex};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_write::trigger::{DROP, QUICK_ADD};
use fs_components_core::form_write::{
    FormWrite, Gesture, NoWrite, WriteArg, WriteDone, WriteRefused,
};
use fs_components_core::states::Load;
use fs_components_fields::KeysAsText;
use fs_components_kanban::{Answer, BoardEvents, BoardSource, MemoryBoard, MoveRequest, NewCard};
use fs_components_kanban::{Card, Column, FormEvents, FormSource, Operation};

/// One call of the port as the board made it: `ask` / `fire`, the trigger, the entry, the record, the names of the arguments.
type Call = (
    &'static str,
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
);

#[derive(Default)]
struct Spy(Mutex<Vec<Call>>);

impl Spy {
    fn note(&self, what: &'static str, g: Gesture<'_>, record: Option<&str>, args: &[WriteArg]) {
        self.0.lock().unwrap().push((
            what,
            g.trigger.to_owned(),
            g.entry.map(str::to_owned),
            record.map(str::to_owned),
            args.iter().map(|a| a.name.clone()).collect(),
        ));
    }
    fn calls(&self) -> Vec<Call> {
        self.0.lock().unwrap().clone()
    }
}

impl FormWrite for Spy {
    fn can(&self, _: &str, _: Option<&str>) -> bool {
        false
    }
    fn send(&self, _: &str, _: Option<&str>, _: &[WriteArg]) -> Result<WriteDone, WriteRefused> {
        panic!("a gesture must not go over a channel name");
    }
    fn may_fire(&self, g: Gesture<'_>, record: Option<&str>) -> Result<(), WriteRefused> {
        self.note("ask", g, record, &[]);
        Ok(())
    }
    fn fire(
        &self,
        g: Gesture<'_>,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.note("fire", g, record, args);
        Ok(WriteDone::default())
    }
}

fn board() -> MemoryBoard {
    let mut todo = Column::new("todo", "Todo");
    todo.cards.push(Card::new("c1", "c1"));
    MemoryBoard::new(vec![todo, Column::new("doing", "Doing")])
}

fn events(spy: &Arc<Spy>) -> FormEvents {
    FormEvents::new(spy.clone(), Arc::new(board()), Arc::new(KeysAsText))
}

fn input() -> FormInput {
    FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![FormRow::of("c1", &[("title", "c1"), ("column", "todo")])],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn call(what: &'static str, trigger: &str, record: Option<&str>, args: &[&str]) -> Call {
    (
        what,
        trigger.to_owned(),
        None,
        record.map(str::to_owned),
        args.iter().map(|a| (*a).to_owned()).collect(),
    )
}

#[test]
fn a_move_fires_the_gesture_drop_for_the_card() {
    let spy = Arc::new(Spy::default());
    let req = MoveRequest::new("c1", "todo", "doing", 0, None);
    assert_eq!(events(&spy).moved(&req), Answer::Accepted);
    assert_eq!(
        spy.calls(),
        vec![
            call("ask", DROP, Some("c1"), &[]),
            call("fire", DROP, Some("c1"), &["column"]),
        ]
    );
}

#[test]
fn a_new_card_fires_the_gesture_quick_add_without_a_record() {
    let spy = Arc::new(Spy::default());
    let card = NewCard::new("todo", "Probe", false);
    assert_eq!(events(&spy).new_card(&card), Answer::Accepted);
    assert_eq!(
        spy.calls(),
        vec![
            call("ask", QUICK_ADD, None, &[]),
            call("fire", QUICK_ADD, None, &["title", "column"]),
        ]
    );
}

#[test]
fn both_gestures_are_asked_before_they_are_fired_and_by_the_same_words() {
    let spy = Arc::new(Spy::default());
    let src = FormSource::new(input(), spy.clone(), Arc::new(KeysAsText));
    assert!(src.supports(Operation::Move) && src.supports(Operation::AddCard));
    assert!(src.locked("c1").is_none());
    assert_eq!(
        spy.calls(),
        vec![
            call("ask", DROP, None, &[]),
            call("ask", QUICK_ADD, None, &[]),
            call("ask", DROP, Some("c1"), &[]),
        ]
    );
}

#[test]
fn a_host_without_gestures_locks_the_board_with_its_own_sentence() {
    let src = FormSource::new(input(), Arc::new(NoWrite), Arc::new(KeysAsText));
    assert!(!src.supports(Operation::Move) && !src.supports(Operation::AddCard));
    let why = src.locked("c1").expect("a host without gestures locks");
    assert_eq!(why.code, "gesture-unbound");
    assert_eq!(why.said(), "kanban.refused.gesture-unbound");
    let ev = FormEvents::new(Arc::new(NoWrite), Arc::new(board()), Arc::new(KeysAsText));
    let req = MoveRequest::new("c1", "todo", "doing", 0, None);
    assert!(matches!(
        ev.moved(&req),
        Answer::Refused(r) if r.said() == "kanban.refused.gesture-unbound"
    ));
    assert!(matches!(
        ev.new_card(&NewCard::new("todo", "x", false)),
        Answer::Refused(r) if r.said() == "kanban.refused.gesture-unbound"
    ));
}
