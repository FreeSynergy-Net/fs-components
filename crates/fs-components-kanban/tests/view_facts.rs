//! O8: the facts line of a card is drawn (`kb-facts-<key>`), inside the card, with the width of its content; a card without
//! facts has no such row. A card that silently left its facts out would lose data without a word.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_kanban::*;
use fs_components_styles::StyleId;
use host::{H, W, board, host};
use ui::{layout, serial};

fn with_facts(values: &[&str]) -> std::sync::Arc<host::Host> {
    let h = host(&Operation::ALL);
    {
        let mut cols = h.cols.lock().unwrap();
        let card = &mut cols[0].cards[0];
        for v in values {
            *card = card
                .clone()
                .fact(FieldSlot::new("field-integer", "label.count", v))
                .expect("room");
        }
    }
    h
}

/// The width of each fact in the row of card `a` (one element per fact), and the height of the row.
fn facts(values: &[&str], style: StyleId) -> Option<(Vec<f32>, f32)> {
    let h = with_facts(values);
    let tree = layout(board(&h, style).build(W, H), W, H);
    assert!(tree.query_by_id("kb-facts-b").is_none(), "no facts, no row");
    let row = tree.query_by_id("kb-facts-a")?;
    let r = tree.get_absolute_bounds(row)?;
    let card = tree.get_absolute_bounds(tree.query_by_id("kb-box-a")?)?;
    assert!(
        r.x >= card.x && r.x + r.width <= card.x + card.width + 0.5,
        "{style:?}: the row is inside its card"
    );
    assert!(r.y >= card.y && r.y + r.height <= card.y + card.height + 0.5);
    let line = tree.query_by_id("kb-facts-line-a")?;
    let widths = tree
        .layout()
        .children(line)
        .into_iter()
        .map(|c| tree.get_absolute_bounds(c).expect("bounds").width)
        .collect();
    Some((widths, r.height))
}

#[test]
fn the_facts_of_a_card_are_drawn_in_every_style() {
    let _g = serial();
    for style in StyleId::ALL {
        let (one, high) = facts(&["7"], style).unwrap_or_else(|| panic!("{style:?}: no facts row"));
        assert!(high > 4.0, "{style:?}: the row has a height ({high})");
        assert_eq!(one.len(), 1, "{style:?}: one element per fact");
        assert!(one[0] > 1.0, "{style:?}: the value is drawn ({one:?})");
        let (two, _) = facts(&["7", "a longer value of the second fact"], style).expect("row");
        assert_eq!(two.len(), 2, "{style:?}");
        assert!(
            two[1] > two[0] + 20.0,
            "{style:?}: the value decides the width ({two:?})"
        );
    }
}

#[test]
fn a_card_without_facts_has_no_facts_row() {
    let _g = serial();
    let h = host(&Operation::ALL);
    let tree = layout(board(&h, StyleId::Admin).build(W, H), W, H);
    assert!(tree.query_by_id("kb-facts-a").is_none());
    assert!(tree.query_by_id("kb-facts-b").is_none());
}

/// A line "+ N ..." (the plus of a button has no number).
fn is_more(t: &str) -> bool {
    t.strip_prefix("+ ")
        .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
}

fn texts(tree: &blinc_layout::prelude::RenderTree) -> Vec<String> {
    tree.iter_nodes()
        .filter_map(|(_, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect()
}

/// P16 and the rule "never cut silently": a card with limit 6 and `n` facts handed in draws 6 and, beyond that, "+ N more".
fn drawn(n: usize) -> (usize, Option<String>, Vec<String>) {
    let _g = serial();
    let h = host(&Operation::ALL);
    {
        let mut cols = h.cols.lock().unwrap();
        let mut card = Card::limited(
            "a",
            "Write the spec",
            fs_components_kanban::Limit::new(6).unwrap(),
        );
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
        cols[0].cards[0] = card;
    }
    let tree = layout(board(&h, StyleId::Admin).build(W, H), W, H);
    assert!(tree.query_by_id("kb-facts-a").is_some(), "the facts row");
    let more = tree.query_by_id("kb-more-a").map(|m| {
        let kids = tree.layout().children(m);
        assert_eq!(kids.len(), 1);
        texts(&tree)
            .into_iter()
            .find(|t| is_more(t))
            .unwrap_or_default()
    });
    let kids = tree
        .layout()
        .children(tree.query_by_id("kb-facts-line-a").expect("the facts line"))
        .len();
    (kids, more, texts(&tree))
}

#[test]
fn a_card_with_a_raised_limit_draws_all_its_facts_and_says_how_many_more() {
    let (drawn_facts, more, _) = drawn(9);
    assert_eq!(drawn_facts, 6, "limit 6: six facts, not four");
    assert_eq!(
        more.as_deref(),
        Some("+ 3 kanban.more_facts"),
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
