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
    let widths = tree
        .layout()
        .children(row)
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
