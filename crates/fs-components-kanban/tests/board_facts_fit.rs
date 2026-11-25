//! The facts line of a card never cuts at the edge: facts that do not fit end in an ellipsis and keep their whole text in a tooltip
//! (`B-card-<key>-flex-<n>`), facts that fit are drawn as they are (no tooltip), and the assignees are drawn in the card with their
//! whole names as tooltips (`B-card-<key>-who-<n>`; more than `MAX_ASSIGNEES` become "+N").
//!
//! Mutants (each makes the named test fail): the line takes no room into account (the old hard clip) ->
//! `long_facts_stay_inside_the_card_and_say_their_whole_text`; short facts get a tooltip as well ->
//! `facts_that_fit_have_no_tooltip`; the assignees are not drawn -> `assignees_are_drawn_with_their_names`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_fields::Look;
use fs_components_kanban::{BoardBench, Card, Column, FieldSlot, Labels, MemoryBoard, card_meta};
use fs_components_styles::StyleId;

const SIZE: (f32, f32) = (1100.0, 640.0);

fn rig(a: Card) -> Rig {
    let mut col = Column::new("todo", "To do");
    col.cards = vec![a];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), Labels::default())
        .look(Look::new(StyleId::Admin, false))
        .id("B");
    Rig::new(bench, SIZE.0, SIZE.1)
}

fn card(values: &[&str], assignees: &[&str]) -> Card {
    let mut card = Card::new("a", "Write the spec");
    for v in values {
        card = card.fact_or_more(FieldSlot::new("field-email", "label.email", v));
    }
    card.assignees = assignees.iter().map(|s| (*s).to_owned()).collect();
    card
}

const LONG: [&str; 3] = [
    "ada.lovelace@example.org",
    "bob.the.builder@example.org",
    "carol@example.org",
];

#[test]
fn long_facts_stay_inside_the_card_and_say_their_whole_text() {
    let rig = rig(card(&LONG, &[]));
    let (cx, _, cw, _) = rig.bounds("B-card-a");
    let node = rig.tree.query_by_id("B-card-a-line").expect("line");
    for child in rig.tree.layout().children(node) {
        let b = rig.tree.get_absolute_bounds(child).expect("bounds");
        assert!(
            b.x + b.width <= cx + cw + 0.5,
            "a piece reaches over the edge of the card: {b:?}"
        );
    }
    assert!(
        (0..3).any(|n| rig.has(&format!("B-card-a-flex-{n}"))),
        "a shortened fact has its tooltip"
    );
}

#[test]
fn facts_that_fit_have_no_tooltip() {
    let rig = rig(card(&["a@b.org", "c@d.org"], &[]));
    assert!(rig.has("B-card-a-line"));
    assert!(!rig.has("B-card-a-flex-0") && !rig.has("B-card-a-flex-1"));
}

#[test]
fn assignees_are_drawn_with_their_names() {
    let rig = rig(card(
        &["a@b.org"],
        &["Ada Lovelace", "Bob", "Carol King", "Dave", "Erin"],
    ));
    for n in 0..4 {
        assert!(rig.has(&format!("B-card-a-who-{n}")), "mark {n}");
    }
    assert!(!rig.has("B-card-a-who-4"));
    assert_eq!(
        card_meta(&card(
            &["a@b.org"],
            &["Ada Lovelace", "Bob", "Carol King", "Dave", "Erin"]
        ))
        .last()
        .map(String::as_str),
        Some("+2")
    );
    let (cx, _, cw, _) = rig.bounds("B-card-a");
    let (x, _, w, _) = rig.bounds("B-card-a-who-3");
    assert!(x + w <= cx + cw + 0.5, "the count is inside the card");
}
