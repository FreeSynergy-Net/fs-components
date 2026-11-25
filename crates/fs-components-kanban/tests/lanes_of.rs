//! The lanes of a board from heads and cards (K1): heads first in their order, unknown card values at the end, a head without a card
//! is an empty lane, a card with several values stands in several lanes.
//!
//! Mutant: the heads are ignored (lanes only from the cards) -> `heads_come_first_in_their_order`,
//! `a_head_without_a_card_is_an_empty_lane` and `a_card_value_without_a_head_is_added_at_the_end` fail (and two more). Run it: in `swimlanes.rs`
//! take the first loop out (`for head in heads`) and run `cargo test -p fs-components-kanban --test lanes_of`: five tests red (checked).
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_kanban::swimlanes::{lanes_of, LaneCard, LaneHead};

fn heads() -> Vec<LaneHead> {
    vec![
        LaneHead::new("ann", "Ann"),
        LaneHead::new("bob", "Bob"),
        LaneHead::new("cy", "Cy"),
    ]
}

fn keys(lanes: &[fs_components_kanban::swimlanes::Lane]) -> Vec<&str> {
    lanes.iter().map(|l| l.key.as_str()).collect()
}

#[test]
fn heads_come_first_in_their_order() {
    let cards = [
        LaneCard::new("1").with("who", &["cy"]),
        LaneCard::new("2").with("who", &["ann"]),
    ];
    let lanes = lanes_of(&heads(), &cards, "who");
    assert_eq!(keys(&lanes), ["ann", "bob", "cy"]);
    assert_eq!(lanes[0].label, "Ann");
    assert_eq!(lanes[0].cards, ["2"]);
    assert_eq!(lanes[2].cards, ["1"]);
}

#[test]
fn a_head_without_a_card_is_an_empty_lane() {
    let lanes = lanes_of(&heads(), &[], "who");
    assert_eq!(keys(&lanes), ["ann", "bob", "cy"]);
    assert!(lanes.iter().all(|l| l.cards.is_empty() && l.headed));
}

#[test]
fn a_card_value_without_a_head_is_added_at_the_end() {
    let cards = [
        LaneCard::new("1").with("who", &["zed"]),
        LaneCard::new("2").with("who", &["ann"]),
        LaneCard::new("3").with("who", &["yan"]),
        LaneCard::new("4").with("who", &["zed"]),
    ];
    let lanes = lanes_of(&heads(), &cards, "who");
    assert_eq!(keys(&lanes), ["ann", "bob", "cy", "zed", "yan"]);
    assert!(!lanes[3].headed);
    assert_eq!(lanes[3].label, "zed");
    assert_eq!(lanes[3].cards, ["1", "4"]);
}

#[test]
fn a_card_with_several_values_stands_in_several_lanes() {
    let cards = [LaneCard::new("1").with("who", &["ann", "cy", "ann"])];
    let lanes = lanes_of(&heads(), &cards, "who");
    assert_eq!(lanes[0].cards, ["1"]);
    assert!(lanes[1].cards.is_empty());
    assert_eq!(lanes[2].cards, ["1"]);
}

#[test]
fn without_heads_the_lanes_follow_the_cards_and_other_fields_are_not_read() {
    let cards = [
        LaneCard::new("1").with("who", &["b"]).with("prio", &["x"]),
        LaneCard::new("2").with("who", &["a"]),
        LaneCard::new("3").with("who", &[" "]),
        LaneCard::new("4"),
    ];
    let lanes = lanes_of(&[], &cards, "who");
    assert_eq!(keys(&lanes), ["b", "a"]);
    assert_eq!(keys(&lanes_of(&[], &cards, "prio")), ["x"]);
}

#[test]
fn a_head_named_twice_counts_once() {
    let h = vec![LaneHead::new("a", "First"), LaneHead::new("a", "Second")];
    let lanes = lanes_of(&h, &[], "who");
    assert_eq!(lanes.len(), 1);
    assert_eq!(lanes[0].label, "First");
}
