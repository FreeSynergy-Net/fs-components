//! SW-2: the board over the write port: gestures become calls of `move` and `create`; `can` = false sends nothing; a refusal comes
//! back as its code.
//!
//! Mutants (each makes the named test fail): the index is left out of the args -> `a_move_sends_the_column_id_and_the_place`; `can`
//! is not asked (always sent) -> `nothing_is_sent_while_the_channel_may_not_be_used`; the column label is sent instead of the id ->
//! `a_move_sends_the_column_id_and_the_place`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use std::sync::{Arc, Mutex};

use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_kanban::*;

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

#[test]
fn a_move_sends_the_column_id_and_the_place() {
    let f = fake(true);
    let ev = FormEvents::new(f.clone());
    assert_eq!(
        ev.move_card_at("c1", "todo", "doing", Some(1)),
        Answer::Accepted
    );
    assert_eq!(
        ev.move_card_at("c1", "todo", "doing", None),
        Answer::Accepted
    );
    let sent = f.sent.lock().unwrap();
    assert_eq!(
        sent[0],
        "move Some(\"c1\") [WriteArg { name: \"column\", value: Text(\"doing\") }, WriteArg { name: \"index\", value: Number(1.0) }]"
    );
    assert!(
        !sent[1].contains("index"),
        "no place = at the end: {}",
        sent[1]
    );
}

#[test]
fn nothing_is_sent_while_the_channel_may_not_be_used() {
    let f = fake(false);
    let ev = FormEvents::new(f.clone());
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
    let ev = FormEvents::new(f.clone());
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
