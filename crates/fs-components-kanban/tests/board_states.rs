//! The states of a slot on the board workbench: a hidden or missing fact shows the plan's reason on the card line and an unset one
//! takes no room; a hidden element is drawn in the frame (as its reason, never as the grey "set").
//!
//! Mutants: `card_meta` keeps empty pieces -> `an_unset_fact_takes_no_room` red; `card_meta` drops a reason ->
//! `a_hidden_fact_shows_its_reason_on_the_card_line` red; the frame skips a slot with a reason -> `a_hidden_element_is_drawn_in_the_frame` red.
//! Pattern: none; tests.

#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_kanban::{
    BoardBench, Card, DetailContent, DetailFrame, FieldSlot, Labels, SlotReason, card_meta,
};

fn why(t: &str) -> SlotReason {
    SlotReason::new("bind-error-field-hidden", t)
}

#[test]
fn a_hidden_fact_shows_its_reason_on_the_card_line() {
    let card = Card::new("k", "K")
        .fact(FieldSlot::hidden(
            "field-integer",
            "l",
            why("hidden: sensitive"),
        ))
        .unwrap()
        .fact(FieldSlot::missing("field-integer", "l", why("not built")))
        .unwrap();
    assert_eq!(card_meta(&card), ["hidden: sensitive", "not built"]);
}

#[test]
fn an_unset_fact_takes_no_room() {
    let card = Card::new("k", "K")
        .fact(FieldSlot::new("field-integer", "l", "7"))
        .unwrap()
        .fact(FieldSlot::unset("field-integer", "l"))
        .unwrap();
    assert_eq!(card_meta(&card), ["7"]);
}

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        DetailFrame::new(format!("Card {card}"))
            .element(
                "Secret",
                FieldSlot::hidden("field-integer", "l", why("hidden: no right")),
            )
            .ok()?
            .element("Note", FieldSlot::unset("field-integer", "l"))
            .ok()
    }
}

#[test]
fn a_hidden_element_is_drawn_in_the_frame() {
    let bench = BoardBench::new(Arc::new(sample()), Labels::default())
        .detail(Arc::new(Detail))
        .id("B");
    let mut rig = Rig::new(bench, 1100.0, 640.0);
    rig.bench.open("a");
    rig.rebuild();
    assert!(rig.has("B-detail-element-0") && rig.has("B-detail-element-1"));
    // the reason has its own text node; the unset element shows the grey "set" instead
    assert!(rig.has("B-detail-element-0-reason"));
    assert!(!rig.has("B-detail-element-1-reason"));
}

#[test]
fn the_state_decides_not_the_text() {
    // mutant: `card_meta` filters by empty text instead of by state -> this test is red.
    // A reason without a sentence is said by its code (never an empty piece, never gone); a value whose text is empty is a value and
    // keeps its piece; only `Unset` takes no room.
    let card = Card::new("k", "K")
        .fact(FieldSlot::hidden(
            "field-integer",
            "l",
            SlotReason::new("bind-error-field-hidden", ""),
        ))
        .unwrap()
        .fact(FieldSlot::new("field-integer", "l", ""))
        .unwrap()
        .fact(FieldSlot::unset("field-integer", "l"))
        .unwrap();
    assert_eq!(card_meta(&card), ["bind-error-field-hidden", ""]);
}
