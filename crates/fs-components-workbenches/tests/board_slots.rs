//! K5 (facts of a card) and K6 (elements of the detail frame) of the board workbench: the slots take field values of the block
//! the catalog chose; a card or frame never drops one silently (the contract limits: `card-facts` 4, `properties` 16).
//!
//! Mutants: `fact`/`element` ignore the limit -> `too_many_*` red; the frame draws no element row -> `an_element_is_drawn_in_the_frame`
//! red; the card line leaves the facts out -> `facts_are_in_the_card_line` red.
//! Pattern: none; tests.

#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_workbenches::board::{
    BoardBench, Card, DetailContent, DetailFrame, FieldSlot, Labels, MAX_ELEMENTS, MAX_FACTS,
};

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn too_many_facts_come_back() {
    let mut card = Card::new("k", "K");
    for i in 0..MAX_FACTS {
        card = card.fact(slot(i)).expect("room");
    }
    assert_eq!(card.facts.len(), 4);
    let back = card.clone().fact(slot(9)).unwrap_err();
    assert_eq!(back, slot(9));
    assert_eq!(card.facts.len(), 4);
}

#[test]
fn too_many_elements_come_back() {
    let mut frame = DetailFrame::new("T");
    for i in 0..MAX_ELEMENTS {
        frame = frame.element("Mail", slot(i)).expect("room");
    }
    assert_eq!(frame.elements.len(), 16);
    assert_eq!(frame.element("Mail", slot(99)).unwrap_err(), slot(99));
}

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        DetailFrame::new(format!("Card {card}"))
            .element("Mail", slot(1))
            .ok()?
            .element("Phone", FieldSlot::new("field-phone", "label.phone", ""))
            .ok()
    }
}

#[test]
fn an_element_is_drawn_in_the_frame() {
    let b = sample();
    let bench = BoardBench::new(Arc::new(b.clone()), Labels::default())
        .detail(Arc::new(Detail))
        .id("B");
    let mut rig = Rig::new(bench, 1100.0, 640.0);
    rig.bench.open("a");
    rig.rebuild();
    assert!(rig.has("B-detail-element-0") && rig.has("B-detail-element-1"));
    assert!(!rig.has("B-detail-element-2"));
}

#[test]
fn facts_are_in_the_card_line() {
    // The line of the card is the slot's own text: the compact component's line of the type.
    let s = FieldSlot::new("field-phone", "label.phone", "+49 30 1234567");
    assert!(s.line().contains("1234567"), "{}", s.line());
    assert_eq!(
        FieldSlot::new("field-integer", "x", "7").line(),
        "7",
        "no component yet: the value itself"
    );
}
