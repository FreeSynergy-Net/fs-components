//! K5 (facts of a card) and K6 (elements of the detail frame) of the board workbench: the slots take field values of the block
//! the catalog chose; a card or frame never drops one silently (a setting of the instance: facts 4, elements 32 by default).
//!
//! Mutants: `fact`/`element` ignore the limit -> `too_many_*` red; the frame draws no element row -> `an_element_is_drawn_in_the_frame`
//! red; the card line leaves the facts out -> `facts_are_in_the_card_line` red.
//! Pattern: none; tests.

#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_kanban::{
    BoardBench, Card, DEFAULT_ELEMENTS, DEFAULT_FACTS, DetailContent, DetailFrame, FieldSlot,
    Labels, card_meta,
};

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn too_many_facts_come_back() {
    let mut card = Card::new("k", "K");
    for i in 0..DEFAULT_FACTS.get() {
        card = card.fact(slot(i)).expect("room");
    }
    assert_eq!(card.facts().len(), 4);
    let back = card.clone().fact(slot(9)).unwrap_err();
    assert_eq!(back, slot(9));
    assert_eq!(card.facts().len(), 4);
}

#[test]
fn too_many_elements_come_back() {
    let mut frame = DetailFrame::new("T");
    for i in 0..DEFAULT_ELEMENTS.get() {
        frame = frame.element("Mail", slot(i)).expect("room");
    }
    assert_eq!(frame.elements().len(), 32);
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

#[test]
fn the_card_line_has_the_facts_between_subtitle_and_due() {
    let mut card = Card::new("k", "K")
        .fact(FieldSlot::new("field-phone", "l", "+49 30 1234567"))
        .unwrap()
        .fact(FieldSlot::new("field-integer", "l", "7"))
        .unwrap();
    card.subtitle = Some("sub".into());
    card.assignees = vec!["Ada".into()];
    let line = card_meta(&card);
    assert_eq!(line.len(), 4);
    assert_eq!(line[0], "sub");
    assert!(line[1].contains("1234567"));
    assert_eq!((line[2].as_str(), line[3].as_str()), ("7", "A"));
    assert!(card_meta(&Card::new("k", "K")).is_empty());
}

struct Big(usize);

impl DetailContent for Big {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        let mut frame = DetailFrame::limited(
            format!("Card {card}"),
            fs_components_kanban::Limit::new(40)?,
        );
        for i in 0..self.0 {
            frame = frame.element_or_more(format!("Field {i}"), slot(i));
        }
        Some(frame)
    }
}

/// A line "+ N ..." (the plus of a button has no number).
fn is_more(t: &str) -> bool {
    t.strip_prefix("+ ")
        .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
}

fn texts(rig: &Rig) -> Vec<String> {
    rig.tree
        .iter_nodes()
        .filter_map(|(_, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect()
}

fn big(n: usize) -> Rig {
    let bench = BoardBench::new(Arc::new(sample()), Labels::default())
        .detail(Arc::new(Big(n)))
        .id("B");
    let mut rig = Rig::new(bench, 1100.0, 640.0);
    rig.bench.open("a");
    rig.rebuild();
    rig
}

/// P15: a frame with limit 40 and 45 elements draws 40 (not sixteen) and says "+ 5 more": nothing is cut silently.
#[test]
fn a_frame_with_a_raised_limit_draws_all_its_elements_and_says_how_many_more() {
    let rig = big(45);
    for i in [0, 15, 16, 31, 39] {
        assert!(
            rig.has(&format!("B-detail-element-{i}")),
            "element {i} is drawn"
        );
    }
    assert!(!rig.has("B-detail-element-40"));
    assert!(rig.has("B-detail-more"));
    assert!(
        texts(&rig).iter().any(|t| t == "+ 5 more"),
        "{:?}",
        texts(&rig)
            .iter()
            .filter(|t| is_more(t))
            .collect::<Vec<_>>()
    );
}

#[test]
fn exactly_the_limit_has_no_more_line() {
    let rig = big(40);
    assert!(rig.has("B-detail-element-39") && !rig.has("B-detail-element-40"));
    assert!(!rig.has("B-detail-more"));
    assert!(!texts(&rig).iter().any(|t| is_more(t)));
    drop(rig); // the rig holds the serial lock
    let few = big(3);
    assert!(!few.has("B-detail-more"));
}

/// The word of "+ N more" is in both languages (MF2, `board-more`).
#[test]
fn the_more_word_is_in_both_languages() {
    for s in fs_components_kanban::TEXTS {
        assert!(s.text.contains("board-more ="), "{}", s.language);
    }
}
