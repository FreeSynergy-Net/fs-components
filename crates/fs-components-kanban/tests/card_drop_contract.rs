//! What a move sends and what the contract declares for the gesture agree: every argument the board's move sends is an argument of
//! the slot `drop` of the block `board` (contract 0.5.0: the drop happens at the CONTAINER, its args are the `move.writes` of the form;
//! one place for the trigger and its arguments). The reverse is not asked: `lane` is declared for the drop of a card between lanes and
//! is not sent by a move yet (P5). The list of trigger words of the catalog is the list of the core (one truth, two crates).
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

mod common;

use std::sync::{Arc, Mutex};

use fs_components_catalog::BlockCatalog;
use fs_components_core::form_write::{FormWrite, Gesture, WriteArg, WriteDone, WriteRefused};
use fs_components_fields::KeysAsText;
use fs_components_kanban::{BoardEvents, Card, Column, FormEvents, MemoryBoard, MoveRequest};

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 6] = [
    (
        "dialogs.toml",
        include_str!("../../../contract/blocks/dialogs.toml"),
    ),
    (
        "fields.toml",
        include_str!("../../../contract/blocks/fields.toml"),
    ),
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
    fn may_fire(&self, gesture: Gesture<'_>, record: Option<&str>) -> Result<(), WriteRefused> {
        if self.can(common::channel_of(gesture), record) {
            Ok(())
        } else {
            Err(WriteRefused::new("locked"))
        }
    }

    fn fire(
        &self,
        gesture: Gesture<'_>,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.send(common::channel_of(gesture), record, args)
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
fn the_arguments_of_a_move_are_declared_by_the_drop_of_the_board() {
    let declared = BlockCatalog::read(CONTRACT, &FILES)
        .unwrap()
        .block("board")
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
            "`{name}` is not declared by board.drop: {declared:?}"
        );
    }
}

#[test]
fn the_card_widget_names_what_the_contract_declares() {
    use fs_components_widgets::card::{ACTIVATE, MENU};
    let cat = BlockCatalog::read(CONTRACT, &FILES).unwrap();
    let card = cat.block("card").unwrap();
    assert_eq!(
        card.widget.as_deref(),
        Some("Card"),
        "block and widget agree"
    );
    // the triggers the component tells are exactly the command slots of the block, no more, no fewer
    let mut slots: Vec<&str> = card.accepts.iter().map(|s| s.name.as_str()).collect();
    slots.sort_unstable();
    let mut said = [ACTIVATE, MENU];
    said.sort_unstable();
    assert_eq!(slots, said);
}

#[test]
fn the_drop_target_of_the_card_widget_has_the_arguments_of_the_board_drop() {
    use fs_components_widgets::card::{DROP, Target};
    let cat = BlockCatalog::read(CONTRACT, &FILES).unwrap();
    // the card widget tells the word, the container declares the arguments (one place: the contract)
    let mut declared = cat.block("board").unwrap().slot(DROP).unwrap().args.clone();
    declared.sort();
    let mut have: Vec<String> = Target::ARGS.iter().map(|s| (*s).to_owned()).collect();
    have.sort();
    assert_eq!(declared, have);
    // and no card slot named drop: the card has no arguments of a form
    assert!(cat.block("card").unwrap().slot(DROP).is_none());
}

#[test]
fn the_trigger_words_of_the_catalog_are_those_of_the_core() {
    use fs_components_core::form_write::trigger;
    assert_eq!(fs_components_catalog::TRIGGERS, trigger::ALL);
}
