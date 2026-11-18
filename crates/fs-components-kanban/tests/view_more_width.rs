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

type Span = (f32, f32);
/// (the line clips, the note clips, the box of the TEXT of the note, its number of characters)
type Flags = (bool, bool, Span, usize);

/// (card right, line (left, right), note (left, right)) of the card `a` with `n` long facts under `limit`.
fn measured(limit: usize, n: usize) -> (f32, Span, Option<Span>) {
    let (card_right, line, note, _) = measured_all(limit, n, false);
    (card_right, line, note)
}

/// Does the element `id` clip what is wider than itself?
fn clips(tree: &blinc_layout::prelude::RenderTree, id: &str) -> bool {
    let node = tree.query_by_id(id).expect("the element");
    tree.iter_nodes()
        .find(|(i, _)| *i == node)
        .map(|(_, n)| n.props.clips_content)
        .expect("a render node")
}

/// Like `measured`, plus (the line clips, the note clips, the right edge of the TEXT of the note).
fn measured_all(limit: usize, n: usize, short: bool) -> (f32, Span, Option<Span>, Flags) {
    let h = host(&Operation::ALL);
    {
        let mut cols = h.cols.lock().unwrap();
        let mut card = Card::limited("a", "Write the spec", Limit::new(limit).unwrap());
        for i in 0..n {
            card = card.fact_or_more(FieldSlot::new(
                "field-email",
                "label.email",
                &if short {
                    format!("{i}")
                } else {
                    format!("firstname.lastname{i}@example.org")
                },
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
    let text = tree
        .iter_nodes()
        .filter_map(|(id, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t)
                if t.content
                    .strip_prefix("+ ")
                    .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit())) =>
            {
                tree.get_absolute_bounds(id)
                    .map(|b| ((b.x, b.x + b.width), t.content.chars().count()))
            }
            _ => None,
        })
        .next()
        .unwrap_or(((0.0, 0.0), 0));
    let note = edges("kb-more-a");
    let flags = (
        clips(&tree, "kb-facts-line-a"),
        note.is_some() && clips(&tree, "kb-more-a"),
        text.0,
        text.1,
    );
    (
        card_right,
        edges("kb-facts-line-a").expect("the facts line"),
        note,
        flags,
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
fn the_line_is_clipped_and_the_note_is_not() {
    let _g = serial();
    for (limit, n) in CASES {
        let (_, _, note, (line_clips, note_clips, text, _)) = measured_all(limit, n, false);
        let text_right = text.1;
        let note = note.expect("the note");
        assert!(
            line_clips,
            "limit {limit}, {n} facts: the long line must be clipped"
        );
        assert!(
            !note_clips,
            "limit {limit}, {n} facts: the note is never clipped"
        );
        assert!(
            text_right <= note.1 + 0.5,
            "limit {limit}, {n} facts: the text of the note ends at {text_right}, its box at {}",
            note.1
        );
    }
}

/// A1 (S2): the note keeps the width of its text: its text is as wide as its words need and its box is not narrower than the text.
#[test]
fn the_note_keeps_its_width() {
    let _g = serial();
    for (limit, n) in CASES {
        let (_, _, note, (_, _, text, chars)) = measured_all(limit, n, false);
        let note = note.expect("the note");
        assert!(
            text.1 - text.0 >= 4.0 * chars as f32,
            "limit {limit}, {n} facts: {chars} characters are {} px wide",
            text.1 - text.0
        );
        assert!(
            note.1 - note.0 >= text.1 - text.0 - 0.5,
            "limit {limit}, {n} facts: the box of the note ({}) is narrower than its text ({})",
            note.1 - note.0,
            text.1 - text.0
        );
    }
}

/// A1 (S7): with SHORT facts the note still stands at the right edge of the card (the line takes the room that is left).
#[test]
fn the_note_is_at_the_right_edge_with_short_facts() {
    let _g = serial();
    for (limit, n) in [(1, 3), (2, 3), (4, 6)] {
        let (card_right, line, note, _) = measured_all(limit, n, true);
        let note = note.expect("the note");
        assert!(
            card_right - note.1 <= 24.0,
            "limit {limit}, {n} short facts: the note is {} px from the right edge",
            card_right - note.1
        );
        assert!(line.1 <= note.0 + 0.5, "the line reaches {}", line.1);
    }
}

#[test]
fn no_note_within_the_limit() {
    let _g = serial();
    for (limit, n) in [(4, 4), (6, 3), (6, 6)] {
        assert!(measured(limit, n).2.is_none(), "limit {limit}, {n} facts");
    }
}
