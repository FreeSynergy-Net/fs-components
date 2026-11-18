//! A fact without a value (`SlotState::Unset`) takes no room on the kanban card, as on the card line of the board workbench: it is not
//! drawn, and a card whose facts are all unset has no facts row (and no height for it). A hidden fact still shows its reason.
//!
//! Mutants (each makes the named test fail): the card draws every fact, `Unset` ones too (`c.shown_facts()` -> `c.facts()` in the view)
//! -> `an_unset_fact_is_not_drawn`; the height counts a facts row for unset facts (`has_facts_line` -> `!facts().is_empty()`) ->
//! `a_card_with_only_unset_facts_has_no_facts_row_and_no_height`; `shown_facts` keeps hidden facts out -> `a_hidden_fact_is_drawn_as_its_reason`; Y5 `has_facts_line` without `more_facts > 0` ->
//! `a_card_full_of_unset_facts_still_says_how_many_more_did_not_fit`; `fact_or_more` does not count an unset fact against the limit ->
//! `an_unset_fact_keeps_its_place_in_the_limit`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_kanban::*;
use fs_components_styles::{StyleId, style};
use host::{H, W, board, host};
use ui::{layout, serial};

fn with(facts: Vec<FieldSlot>) -> std::sync::Arc<host::Host> {
    let h = host(&Operation::ALL);
    {
        let mut cols = h.cols.lock().unwrap();
        let mut card = cols[0].cards[0].clone();
        for f in facts {
            card = card.fact(f).expect("room");
        }
        cols[0].cards[0] = card;
    }
    h
}

fn value(v: &str) -> FieldSlot {
    FieldSlot::new("field-integer", "label.count", v)
}

fn unset() -> FieldSlot {
    FieldSlot::unset("field-integer", "label.count")
}

fn kids_of_facts_row(h: &std::sync::Arc<host::Host>) -> Option<usize> {
    let tree = layout(board(h, StyleId::Admin).build(W, H), W, H);
    let row = tree.query_by_id("kb-facts-line-a")?;
    Some(tree.layout().children(row).len())
}

#[test]
fn an_unset_fact_is_not_drawn() {
    let _g = serial();
    let h = with(vec![value("7"), unset(), value("9")]);
    assert_eq!(
        kids_of_facts_row(&h),
        Some(2),
        "two values, the unset one has no place"
    );
    let c = h.cols.lock().unwrap()[0].cards[0].clone();
    assert_eq!(
        c.facts().len(),
        3,
        "it stays in the model (it counts against the limit)"
    );
    assert_eq!(c.shown_facts().count(), 2);
}

#[test]
fn a_card_with_only_unset_facts_has_no_facts_row_and_no_height() {
    let _g = serial();
    let h = with(vec![unset(), unset()]);
    assert_eq!(kids_of_facts_row(&h), None);
    let bare = host(&Operation::ALL);
    let (a, b) = (
        h.cols.lock().unwrap()[0].cards[0].clone(),
        bare.cols.lock().unwrap()[0].cards[0].clone(),
    );
    let s = style(StyleId::Admin);
    assert!(!a.has_facts_line());
    assert_eq!(
        fs_components_kanban::layout::card_height(&a, s),
        fs_components_kanban::layout::card_height(&b, s),
        "no row, no room"
    );
}

#[test]
fn a_hidden_fact_is_drawn_as_its_reason() {
    let _g = serial();
    let why = SlotReason::new("bind-error-field-hidden", "hidden: sensitive");
    let h = with(vec![
        unset(),
        FieldSlot::hidden("field-integer", "label.count", why),
    ]);
    assert_eq!(
        kids_of_facts_row(&h),
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
/// did not fit: the facts line, its height and the piece "+ N more" must stay (the value is not lost silently).
#[test]
fn a_card_full_of_unset_facts_still_says_how_many_more_did_not_fit() {
    let _g = serial();
    let h = host(&Operation::ALL);
    let bare = limited(2, vec![]);
    let card = limited(2, vec![unset(), unset(), value("7")]);
    assert_eq!((card.facts().len(), card.more_facts()), (2, 1));
    assert_eq!(
        card.shown_facts().count(),
        0,
        "nothing of the shown facts is drawn"
    );
    assert!(card.has_facts_line(), "the piece '+ 1 more' needs its line");
    let s = style(StyleId::Admin);
    assert!(
        fs_components_kanban::layout::card_height(&card, s)
            > fs_components_kanban::layout::card_height(&bare, s),
        "and its room"
    );
    h.cols.lock().unwrap()[0].cards[0] = card;
    let tree = layout(board(&h, StyleId::Admin).build(W, H), W, H);
    assert!(
        tree.query_by_id("kb-facts-a").is_some(),
        "the line is drawn"
    );
    let more = tree.query_by_id("kb-more-a").expect("the piece is drawn");
    assert_eq!(tree.layout().children(more).len(), 1);
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
