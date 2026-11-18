//! B1 (never cut silently, the card line of the board): facts beyond the limit of a card say "+ N more" in the grey line, in the
//! words of the language (MF2 `board-more`, en with the `one` branch, and de); exactly the limit says nothing.
//!
//! Mutants: `card_meta` ignores `more_facts()` -> `the_card_line_says_how_many_facts_are_left_out`, `the_board_draws_the_more_piece_in_the_card_line`;
//! the count is off by one or the word is wrong -> the same; the German text -> `the_words_of_more_in_german` (Q4); the English
//! `one` branch -> `the_words_of_more_in_english` (Q5).
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_words::{Locale, Localized};
use fs_components_workbenches::board::{
    BoardBench, Card, Column, FieldSlot, Labels, Limit, MemoryBoard, card_meta,
};

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

fn card(limit: usize, facts: usize) -> Card {
    let mut c = Card::limited("k", "K", Limit::new(limit).unwrap());
    for i in 0..facts {
        c = c.fact_or_more(slot(i));
    }
    c
}

fn german() -> Labels {
    Labels::for_locale(&Locale::parse("de").unwrap())
}

#[test]
fn the_card_line_says_how_many_facts_are_left_out() {
    let en = Labels::default();
    // limit 4, 6 facts: 4 facts and "+ 2 more"
    let line = card_meta(&card(4, 6), &en);
    assert_eq!(line.len(), 5, "{line:?}");
    assert!(line[3].contains("a3@example.org"), "{line:?}");
    assert_eq!(line[4], "+ 2 more");
    // one beyond the limit: the `one` branch
    let line = card_meta(&card(4, 5), &en);
    assert_eq!((line.len(), line[4].as_str()), (5, "+ 1 more"), "{line:?}");
    // exactly the limit, and fewer: no piece
    for facts in [0, 3, 4] {
        let line = card_meta(&card(4, facts), &en);
        assert_eq!(line.len(), facts, "{facts}: {line:?}");
        assert!(line.iter().all(|p| !p.starts_with("+ ")), "{line:?}");
    }
    // the piece stands right after the facts, before the due text and the assignee
    let mut c = card(2, 3);
    c.assignee = Some("Ada".into());
    let line = card_meta(&c, &en);
    assert_eq!(
        (line[2].as_str(), line[3].as_str()),
        ("+ 1 more", "Ada"),
        "{line:?}"
    );
    // in German
    let line = card_meta(&card(4, 9), &german());
    assert_eq!(line[4], "+ 5 weitere", "{line:?}");
}

#[test]
fn the_words_of_more_in_english() {
    let en = Labels::default();
    assert_eq!(en.more(1), "+ 1 more", "the `one` branch");
    assert_eq!(en.more(2), "+ 2 more");
    assert_eq!(en.more(5), "+ 5 more");
}

#[test]
fn the_words_of_more_in_german() {
    let de = german();
    assert_eq!(de.more(1), "+ 1 weitere", "the `one` branch");
    assert_eq!(de.more(5), "+ 5 weitere");
    assert_eq!(de.more(12), "+ 12 weitere");
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

fn board_with(c: Card) -> Rig {
    let mut col = Column::new("todo", "To do");
    col.cards = vec![c];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), Labels::default()).id("B");
    Rig::new(bench, 1100.0, 640.0)
}

#[test]
fn the_board_draws_the_more_piece_in_the_card_line() {
    let rig = board_with(card(4, 6));
    let line = texts(&rig)
        .into_iter()
        .find(|t| t.contains("a0@example.org"))
        .expect("the card line");
    assert!(line.contains("a3@example.org"), "{line}");
    assert!(!line.contains("a4@example.org"), "{line}");
    assert!(line.ends_with("+ 2 more"), "{line}");
    drop(rig);
    let rig = board_with(card(4, 4));
    let line = texts(&rig)
        .into_iter()
        .find(|t| t.contains("a0@example.org"))
        .expect("the card line");
    assert!(!line.contains("more"), "{line}");
}
