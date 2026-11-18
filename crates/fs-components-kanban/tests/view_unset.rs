//! A fact without a value (`SlotState::Unset`) takes no room on the kanban card, as on the card line of the board workbench: it is not
//! drawn, and a card whose facts are all unset has no facts row (and no height for it). A hidden fact still shows its reason.
//!
//! Mutants (each makes the named test fail): the card draws every fact, `Unset` ones too (`c.shown_facts()` -> `c.facts()` in the view)
//! -> `an_unset_fact_is_not_drawn`; the height counts a facts row for unset facts (`has_facts_line` -> `!facts().is_empty()`) ->
//! `a_card_with_only_unset_facts_has_no_facts_row_and_no_height`; `shown_facts` keeps hidden facts out -> `a_hidden_fact_is_drawn_as_its_reason`.
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
    let row = tree.query_by_id("kb-facts-a")?;
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
