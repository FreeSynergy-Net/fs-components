//! A fact without a value (`SlotState::Unset`) takes no room on the card of the one board element (moved from `view_unset.rs`): it is not
//! drawn, and a card whose facts are all unset has no facts line (the card stays `CARD_H` high). A hidden fact still shows its reason.
//!
//! Mutants (each makes the named test fail): the card draws every fact, `Unset` ones too (`shown_facts()` -> `facts()` in the line) ->
//! `an_unset_fact_is_not_drawn`; the line is made for unset facts (`has_line` -> `!facts().is_empty()`) ->
//! `a_card_with_only_unset_facts_has_no_facts_line_and_keeps_its_height`; `shown_facts` keeps hidden facts out ->
//! `a_hidden_fact_is_drawn_as_its_reason`; Y5 `has_facts_line` without `more_facts > 0` ->
//! `a_card_full_of_unset_facts_still_says_how_many_more_did_not_fit`; `fact_or_more` does not count an unset fact against the limit ->
//! `an_unset_fact_keeps_its_place_in_the_limit`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_kanban::board::hit::CARD_H;
use fs_components_kanban::board::{
    BoardBench, Card, Column, FieldSlot, Labels, Limit, MemoryBoard, SlotReason,
};

fn rig(card: Card) -> Rig {
    let mut col = Column::new("todo", "To do");
    col.cards = vec![card];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), Labels::default()).id("B");
    Rig::new(bench, 1100.0, 640.0)
}

fn with(facts: Vec<FieldSlot>) -> Card {
    let mut card = Card::new("a", "Write the spec");
    for f in facts {
        card = card.fact(f).expect("room");
    }
    card
}

fn value(v: &str) -> FieldSlot {
    FieldSlot::new("field-integer", "label.count", v)
}

fn unset() -> FieldSlot {
    FieldSlot::unset("field-integer", "label.count")
}

fn kids_of_facts_line(rig: &Rig) -> Option<usize> {
    let line = rig.tree.query_by_id("B-card-a-line")?;
    Some(rig.tree.layout().children(line).len())
}

#[test]
fn an_unset_fact_is_not_drawn() {
    let card = with(vec![value("7"), unset(), value("9")]);
    assert_eq!(
        kids_of_facts_line(&rig(card.clone())),
        Some(2),
        "two values, the unset one has no place"
    );
    assert_eq!(
        card.facts().len(),
        3,
        "it stays in the model (it counts against the limit)"
    );
    assert_eq!(card.shown_facts().count(), 2);
}

#[test]
fn a_card_with_only_unset_facts_has_no_facts_line_and_keeps_its_height() {
    let card = with(vec![unset(), unset()]);
    assert!(!card.has_facts_line());
    let rig = rig(card);
    assert_eq!(kids_of_facts_line(&rig), None);
    let (_, _, _, h) = rig.bounds("B-card-a");
    assert!(
        (h - CARD_H).abs() < 1.5,
        "no line, the same card height ({h})"
    );
}

#[test]
fn a_hidden_fact_is_drawn_as_its_reason() {
    let why = SlotReason::new("bind-error-field-hidden", "hidden: sensitive");
    let card = with(vec![
        unset(),
        FieldSlot::hidden("field-integer", "label.count", why),
    ]);
    assert_eq!(
        kids_of_facts_line(&rig(card)),
        Some(1),
        "the reason takes a place, the unset fact does not"
    );
}

fn limited(n: usize, facts: Vec<FieldSlot>) -> Card {
    let mut card = Card::limited("a", "Write the spec", Limit::new(n).unwrap());
    for f in facts {
        card = card.fact_or_more(f);
    }
    card
}

/// Y5: the limit counts the slots, so a card whose limit is full of unset facts has nothing to draw but "+ N more" for the value that
/// did not fit: the facts line and the piece "+ N more" must stay (the value is not lost silently).
#[test]
fn a_card_full_of_unset_facts_still_says_how_many_more_did_not_fit() {
    let card = limited(2, vec![unset(), unset(), value("7")]);
    assert_eq!((card.facts().len(), card.more_facts()), (2, 1));
    assert_eq!(
        card.shown_facts().count(),
        0,
        "nothing of the shown facts is drawn"
    );
    assert!(card.has_facts_line(), "the piece '+ 1 more' needs its line");
    let rig = rig(card);
    assert!(rig.has("B-card-a-line"), "the line is drawn");
    let more = rig
        .tree
        .query_by_id("B-card-a-more")
        .expect("the piece is drawn");
    assert_eq!(rig.tree.layout().children(more).len(), 1);
    assert_eq!(kids_of_facts_line(&rig), Some(0), "no fact in it");
}

/// The rule as it is (and said in the docs): an unset fact is counted against the limit, so the third slot of a card of two is "more"
/// even when the first is unset.
#[test]
fn an_unset_fact_keeps_its_place_in_the_limit() {
    let card = limited(2, vec![unset(), value("7"), value("9")]);
    assert_eq!(card.facts().len(), 2);
    assert_eq!(card.more_facts(), 1);
    assert_eq!(card.shown_facts().count(), 1);
    let none = limited(2, vec![value("7"), value("9")]);
    assert_eq!(none.more_facts(), 0, "exactly the limit says nothing");
}
