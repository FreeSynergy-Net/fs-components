//! What a move sends and what the contract declares for the gesture agree: every argument the board's move sends is an argument of
//! the slot `drop` of the block `card` (P1, rule M1: one place for the trigger and its arguments). The reverse is not asked:
//! `lane` is declared for the drop of a card between lanes and is not sent by a move yet (P5).
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use std::sync::{Arc, Mutex};

use fs_components_catalog::BlockCatalog;
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_fields::KeysAsText;
use fs_components_kanban::{BoardEvents, Card, Column, FormEvents, MemoryBoard, MoveRequest};

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 4] = [
    (
        "frames.toml",
        include_str!("../../../contract/blocks/frames.toml"),
    ),
    (
        "general.toml",
        include_str!("../../../contract/blocks/general.toml"),
    ),
    (
        "programs.toml",
        include_str!("../../../contract/blocks/programs.toml"),
    ),
    (
        "widgets.toml",
        include_str!("../../../contract/blocks/widgets.toml"),
    ),
];

#[derive(Default)]
struct Rec(Mutex<Vec<String>>);

impl FormWrite for Rec {
    fn can(&self, _: &str, _: Option<&str>) -> bool {
        true
    }
    fn send(&self, _: &str, _: Option<&str>, args: &[WriteArg]) -> Result<WriteDone, WriteRefused> {
        self.0
            .lock()
            .unwrap()
            .extend(args.iter().map(|a| a.name.clone()));
        Ok(WriteDone::default())
    }
}

#[test]
fn the_arguments_of_a_move_are_declared_by_the_drop_of_the_card() {
    let declared = BlockCatalog::read(CONTRACT, &FILES)
        .unwrap()
        .block("card")
        .unwrap()
        .slot("drop")
        .unwrap()
        .args
        .clone();
    let mut doing = Column::new("doing", "Doing");
    for k in ["a", "b"] {
        doing.cards.push(Card::new(k, k));
    }
    let mut todo = Column::new("todo", "Todo");
    todo.cards.push(Card::new("c1", "c1"));
    let rec = Arc::new(Rec::default());
    let ev = FormEvents::new(
        rec.clone(),
        Arc::new(MemoryBoard::new(vec![todo, doing])),
        Arc::new(KeysAsText),
    );
    // between two neighbours: column, after and before are all sent
    ev.moved(&MoveRequest::new(
        "c1",
        "todo",
        "doing",
        1,
        Some("a".into()),
    ));
    let sent = rec.0.lock().unwrap().clone();
    assert_eq!(sent.len(), 3, "{sent:?}");
    for name in &sent {
        assert!(
            declared.contains(name),
            "`{name}` is not declared by card.drop: {declared:?}"
        );
    }
}
