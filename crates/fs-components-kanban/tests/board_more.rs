//! B1 (never cut silently, the card line of the board): facts beyond the limit of a card say "+ N more" next to the grey line, in the
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
use fs_components_kanban::{
    BoardBench, Card, Column, FieldSlot, Labels, Limit, MemoryBoard, card_meta, card_more,
};
use fs_components_words::{Locale, Localized};

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
    // limit 4, 6 facts: the line holds 4 facts, the note says "+ 2 more" (apart from the line: it is drawn at the right end)
    let line = card_meta(&card(4, 6));
    assert_eq!(line.len(), 4, "{line:?}");
    assert!(line[3].contains("a3@example.org"), "{line:?}");
    assert!(line.iter().all(|p| !p.starts_with("+ ")), "{line:?}");
    assert_eq!(card_more(&card(4, 6), &en).as_deref(), Some("+ 2 more"));
    // one beyond the limit: the `one` branch
    assert_eq!(card_meta(&card(4, 5)).len(), 4);
    assert_eq!(card_more(&card(4, 5), &en).as_deref(), Some("+ 1 more"));
    // exactly the limit, and fewer: no note
    for facts in [0, 3, 4] {
        assert_eq!(card_meta(&card(4, facts)).len(), facts);
        assert_eq!(card_more(&card(4, facts), &en), None, "{facts}");
    }
    // the due text and the assignee stay in the line, after the facts
    let mut c = card(2, 3);
    c.assignees = vec!["Ada".into()];
    let line = card_meta(&c);
    assert_eq!((line.len(), line[2].as_str()), (3, "A"), "{line:?}");
    assert_eq!(card_more(&c, &en).as_deref(), Some("+ 1 more"));
    // in German
    assert_eq!(
        card_more(&card(4, 9), &german()).as_deref(),
        Some("+ 5 weitere")
    );
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
    let all = texts(&rig);
    // each fact is drawn by its own component (K2a), no longer joined into one text
    // a fact that does not fit its share ends in an ellipsis (board_facts_fit.rs), so it is found by its start
    let shown: Vec<_> = all
        .iter()
        .filter(|t| t.starts_with('a') && (t.ends_with('\u{2026}') || t.contains("@example")))
        .collect();
    // four long e-mails do not fit one card: the ones without room are left out and counted in the note (board_facts_fit.rs)
    let left_out = 4 - shown.len();
    assert!(!shown.is_empty() && shown.len() <= 4, "{shown:?} {all:?}");
    assert!(
        shown.iter().all(|t| !t.contains("more")),
        "the note is its own element: {shown:?}"
    );
    assert_eq!(
        all.iter()
            .filter(|t| t.as_str() == format!("+ {} more", 2 + left_out))
            .count(),
        1,
        "{all:?}"
    );
    drop(rig);
    let rig = board_with(card(4, 4));
    let all = texts(&rig);
    assert!(all.iter().all(|t| !t.contains("more")), "{all:?}");
}
