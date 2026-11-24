//! O8, moved to the one board element (from `view_facts.rs`): the facts line of a card is drawn (`B-card-<key>-line`) inside the card, by the
//! field components (`FieldSlot::compact_view`), each with the width of its content; a card without facts has no such line. A card that
//! silently left its facts out would lose data without a word. P16 and the rule "never cut silently": a card with limit 6 and `n` facts
//! draws 6 and, beyond that, "+ N more".
//!
//! Mutants (each makes the named test fail): the line draws no facts -> `the_facts_of_a_card_are_drawn_in_every_style`; the card
//! always gets a line -> `a_card_without_facts_has_no_facts_line`; the limit is not honoured (`fact_or_more` keeps four) ->
//! `a_card_with_a_raised_limit_draws_all_its_facts_and_says_how_many_more`; the note is drawn at the limit ->
//! `exactly_the_limit_has_no_more_line`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_fields::Look;
use fs_components_kanban::board::{
    BoardBench, Card, Column, FieldSlot, Labels, Limit, MemoryBoard,
};
use fs_components_styles::StyleId;

const SIZE: (f32, f32) = (1100.0, 640.0);

/// A board of one column with the cards `a` (the given one) and `b` (plain), drawn in `style`.
fn rig(a: Card, style: StyleId) -> Rig {
    let mut col = Column::new("todo", "To do");
    col.cards = vec![a, Card::new("b", "Buy stamps")];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), Labels::default())
        .look(Look::new(style, false))
        .id("B");
    Rig::new(bench, SIZE.0, SIZE.1)
}

fn with_facts(values: &[&str]) -> Card {
    let mut card = Card::new("a", "Write the spec");
    for v in values {
        card = card
            .fact(FieldSlot::new("field-integer", "label.count", v))
            .expect("room");
    }
    card
}

/// The width of each fact in the line of card `a` (one element per fact), and the height of the line.
fn facts(values: &[&str], style: StyleId) -> Option<(Vec<f32>, f32)> {
    let rig = rig(with_facts(values), style);
    assert!(!rig.has("B-card-b-line"), "no facts, no line");
    if !rig.has("B-card-a-line") {
        return None;
    }
    let (lx, ly, lw, lh) = rig.bounds("B-card-a-line");
    let (cx, cy, cw, ch) = rig.bounds("B-card-a");
    assert!(
        lx >= cx && lx + lw <= cx + cw + 0.5,
        "{style:?}: the line is inside its card"
    );
    assert!(ly >= cy && ly + lh <= cy + ch + 0.5);
    let node = rig.tree.query_by_id("B-card-a-line")?;
    let widths = rig
        .tree
        .layout()
        .children(node)
        .into_iter()
        .map(|c| rig.tree.get_absolute_bounds(c).expect("bounds").width)
        .collect();
    Some((widths, lh))
}

#[test]
fn the_facts_of_a_card_are_drawn_in_every_style() {
    for style in StyleId::ALL {
        let (one, high) =
            facts(&["7"], style).unwrap_or_else(|| panic!("{style:?}: no facts line"));
        assert!(high > 4.0, "{style:?}: the line has a height ({high})");
        assert_eq!(one.len(), 1, "{style:?}: one element per fact");
        assert!(one[0] > 1.0, "{style:?}: the value is drawn ({one:?})");
        let (two, _) = facts(&["7", "a longer value of the second fact"], style).expect("line");
        assert_eq!(two.len(), 2, "{style:?}");
        assert!(
            two[1] > two[0] + 20.0,
            "{style:?}: the value decides the width ({two:?})"
        );
    }
}

#[test]
fn a_card_without_facts_has_no_facts_line() {
    let rig = rig(Card::new("a", "Write the spec"), StyleId::Admin);
    assert!(!rig.has("B-card-a-line"));
    assert!(!rig.has("B-card-b-line"));
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

/// A card with limit 6 and `n` facts handed in: (facts drawn, the note "+ N more", all texts).
fn drawn(n: usize) -> (usize, Option<String>, Vec<String>) {
    let mut card = Card::limited("a", "Write the spec", Limit::new(6).unwrap());
    for i in 0..n {
        card = card.fact_or_more(FieldSlot::new(
            "field-integer",
            "label.count",
            &format!("value {i}"),
        ));
    }
    assert_eq!(
        (card.facts().len(), card.more_facts()),
        (n.min(6), n.saturating_sub(6))
    );
    let rig = rig(card, StyleId::Admin);
    assert!(rig.has("B-card-a-line"), "the facts line");
    let more = rig.tree.query_by_id("B-card-a-more").map(|m| {
        assert_eq!(rig.tree.layout().children(m).len(), 1);
        texts(&rig)
            .into_iter()
            .find(|t| is_more(t))
            .unwrap_or_default()
    });
    let kids = rig
        .tree
        .layout()
        .children(
            rig.tree
                .query_by_id("B-card-a-line")
                .expect("the facts line"),
        )
        .len();
    (kids, more, texts(&rig))
}

#[test]
fn a_card_with_a_raised_limit_draws_all_its_facts_and_says_how_many_more() {
    let (drawn_facts, more, _) = drawn(9);
    assert_eq!(drawn_facts, 6, "limit 6: six facts, not four");
    assert_eq!(
        more.as_deref(),
        Some("+ 3 more"),
        "nine facts, six drawn: + 3 more, with the word"
    );
}

#[test]
fn exactly_the_limit_has_no_more_line() {
    let (drawn_facts, more, all) = drawn(6);
    assert_eq!((drawn_facts, more), (6, None));
    assert!(!all.iter().any(|t| is_more(t)));
    let (fewer, none, _) = drawn(2);
    assert_eq!((fewer, none), (2, None));
}
