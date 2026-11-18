//! A1 (never cut silently, the width of the facts row of a kanban card): the note "+ N ..." is MEASURED inside the card, for long
//! facts (e-mail addresses: six of them are about 520 px wide, the card 279 px) as for short ones, at every raised limit. The facts
//! line gives way (it is clipped at its end), the note keeps its width at the right end.
//!
//! Mutants: the note inside the facts line again / the row without a fixed note (`flex_shrink_0` removed) -> `the_note_stays_inside_the_card`;
//! the note before the facts -> `the_note_is_at_the_right_end`; no clip on the line -> `the_note_stays_inside_the_card`.
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

/// (card right, line (left, right), note (left, right)) of the card `a` with `n` long facts under `limit`.
fn measured(limit: usize, n: usize) -> (f32, (f32, f32), Option<(f32, f32)>) {
    let h = host(&Operation::ALL);
    {
        let mut cols = h.cols.lock().unwrap();
        let mut card = Card::limited("a", "Write the spec", Limit::new(limit).unwrap());
        for i in 0..n {
            card = card.fact_or_more(FieldSlot::new(
                "field-email",
                "label.email",
                &format!("firstname.lastname{i}@example.org"),
            ));
        }
        cols[0].cards[0] = card;
    }
    let tree = layout(board(&h, StyleId::Admin).build(W, H), W, H);
    let edges = |id: &str| {
        let b = tree
            .get_absolute_bounds(tree.query_by_id(id)?)
            .expect("bounds");
        Some((b.x, b.x + b.width))
    };
    let (_, card_right) = edges("kb-box-a").expect("the card");
    (
        card_right,
        edges("kb-facts-line-a").expect("the facts line"),
        edges("kb-more-a"),
    )
}

const CASES: [(usize, usize); 5] = [(4, 5), (4, 6), (6, 7), (6, 9), (12, 20)];

#[test]
fn the_note_stays_inside_the_card() {
    let _g = serial();
    for (limit, n) in CASES {
        let (card_right, line, note) = measured(limit, n);
        let note = note.unwrap_or_else(|| panic!("limit {limit}, {n} facts: no note"));
        assert!(
            note.1 <= card_right + 0.5,
            "limit {limit}, {n} facts: the note ends at {} px, the card at {card_right}",
            note.1
        );
        assert!(line.1 <= card_right + 0.5, "the line ends at {}", line.1);
    }
}

#[test]
fn the_note_is_at_the_right_end() {
    let _g = serial();
    for (limit, n) in CASES {
        let (card_right, line, note) = measured(limit, n);
        let note = note.expect("the note");
        assert!(
            line.1 <= note.0 + 0.5,
            "limit {limit}, {n} facts: the line reaches {} but the note begins at {}",
            line.1,
            note.0
        );
        assert!(
            card_right - note.1 <= 24.0,
            "limit {limit}, {n} facts: the note is {} px from the right edge",
            card_right - note.1
        );
        assert!(
            line.1 - line.0 >= 40.0,
            "the line keeps some room ({line:?})"
        );
    }
}

#[test]
fn no_note_within_the_limit() {
    let _g = serial();
    for (limit, n) in [(4, 4), (6, 3), (6, 6)] {
        assert!(measured(limit, n).2.is_none(), "limit {limit}, {n} facts");
    }
}
