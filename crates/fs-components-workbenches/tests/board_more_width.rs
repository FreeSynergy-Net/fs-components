//! A1 (never cut silently, the width of the card line): "+ N more" is an element of its own at the right end of the grey line of a
//! card and is MEASURED inside the card, for long facts (limit 4, six e-mail facts: the line is about 520 px wide, the card 288 px)
//! as for short ones, in both languages, in a narrow and in a wide window. The line gives way; the note never does.
//!
//! Mutants: the note back inside the clipped text of the line -> `the_more_note_stays_inside_the_card` (its box has no width left
//! or the text is not found); the note without a fixed width (`flex_shrink` instead of `flex_shrink_0`) -> `the_more_note_keeps_its_width`;
//! the note at the left of the line -> `the_more_note_is_at_the_right_end`; the note clipped (`overflow_clip` with a box too
//! narrow) -> `the_more_note_keeps_its_width`; no note on a card within its limit -> `no_note_within_the_limit`.
//!
//! Pattern: none; tests (measure the laid-out tree, never a guessed number).
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_words::{Locale, Localized};
use fs_components_workbenches::board::{
    BoardBench, Card, Column, FieldSlot, Labels, Limit, MemoryBoard,
};

const CARD_W: f32 = 288.0;

fn long_card(limit: usize, facts: usize) -> Card {
    let mut c = Card::limited("k", "K", Limit::new(limit).unwrap());
    for i in 0..facts {
        c = c.fact_or_more(FieldSlot::new(
            "field-email",
            "label.email",
            &format!("firstname.lastname{i}@example.org"),
        ));
    }
    c
}

fn rig(c: Card, labels: Labels, w: f32) -> Rig {
    let mut col = Column::new("todo", "To do");
    col.cards = vec![c];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), labels).id("B");
    Rig::new(bench, w, 640.0)
}

/// The box of the TEXT that says `word`, as laid out: (left, right).
fn text_edges(rig: &Rig, word: &str) -> (f32, f32) {
    let found: Vec<_> = rig
        .tree
        .iter_nodes()
        .filter(|(_, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t) => t.content.starts_with(word),
            _ => false,
        })
        .map(|(id, _)| id)
        .collect();
    assert_eq!(found.len(), 1, "one text starts with {word:?}");
    let b = rig.tree.get_absolute_bounds(found[0]).expect("bounds");
    (b.x, b.x + b.width)
}

/// Does the element `id` clip what is wider than itself? (Else the text of a long line is painted over the note.)
fn clips(rig: &Rig, id: &str) -> bool {
    let node = rig.tree.query_by_id(id).expect("the element");
    rig.tree
        .iter_nodes()
        .find(|(i, _)| *i == node)
        .map(|(_, n)| n.props.clips_content)
        .expect("a render node")
}

fn card_edges(rig: &Rig) -> (f32, f32) {
    let (x, _, w, _) = rig.bounds("B-card-k");
    (x, x + w)
}

fn cases() -> Vec<(&'static str, Labels, usize, usize, f32)> {
    let en = Labels::default();
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    let mut v = Vec::new();
    for w in [340.0, 1100.0, 1700.0] {
        v.push(("+ 2", en.clone(), 4, 6, w));
        v.push(("+ 12", de.clone(), 4, 16, w));
        v.push(("+ 1", en.clone(), 4, 5, w));
    }
    v
}

#[test]
fn the_more_note_stays_inside_the_card() {
    for (word, labels, limit, facts, w) in cases() {
        let rig = rig(long_card(limit, facts), labels, w);
        let (_, card_right) = card_edges(&rig);
        let (_, right) = text_edges(&rig, word);
        assert!(
            right <= card_right + 0.5,
            "{word} at {w}: the note ends at {right}, the card at {card_right}"
        );
    }
}

#[test]
fn the_more_note_keeps_its_width() {
    for (word, labels, limit, facts, w) in cases() {
        let rig = rig(long_card(limit, facts), labels.clone(), w);
        let full = labels.more(facts - limit);
        let (left, right) = text_edges(&rig, word);
        // the text is as wide as its words need (about 5 px a letter at 13 px): never squeezed to a stump
        assert!(
            right - left >= 4.0 * full.len() as f32,
            "{word} at {w}: {full:?} is {} px wide",
            right - left
        );
        let (nl, nr, ..) = {
            let (x, _, bw, _) = rig.bounds("B-card-k-more");
            (x, x + bw, 0)
        };
        assert!(
            nr - nl >= right - left - 0.5,
            "{word} at {w}: the box of the note is narrower than its text"
        );
    }
}

#[test]
fn the_more_note_is_at_the_right_end() {
    for (word, labels, limit, facts, w) in cases() {
        let rig = rig(long_card(limit, facts), labels, w);
        let (_, card_right) = card_edges(&rig);
        let (left, right) = text_edges(&rig, word);
        let (lx, _, lw, _) = rig.bounds("B-card-k-line");
        // the room of the line ends where the note begins; the note ends within one padding of the card edge
        assert!(
            lx + lw <= left + 0.5,
            "{word} at {w}: the line reaches {} but the note begins at {left}",
            lx + lw
        );
        assert!(
            card_right - right <= 24.0,
            "{word} at {w}: the note is {} px from the right edge",
            card_right - right
        );
        // the line still has room of its own: at least a third of the card
        assert!(lw >= CARD_W / 3.0, "{word} at {w}: the line has {lw} px");
    }
}

#[test]
fn the_line_is_clipped_and_the_note_is_not() {
    let rig = rig(long_card(4, 6), Labels::default(), 1100.0);
    assert!(
        clips(&rig, "B-card-k-line"),
        "the long line must be clipped at its box, else it is painted over the note"
    );
    assert!(!clips(&rig, "B-card-k-more"), "the note is never clipped");
}

/// A1 (S7): with SHORT facts the note still stands at the right edge of the card (the line takes the room that is left), in
/// both languages and in a narrow and a wide window.
#[test]
fn the_note_is_at_the_right_edge_with_short_facts() {
    let en = Labels::default();
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    for (limit, facts, word, labels) in [(1, 3, "+ 2", &en), (2, 3, "+ 1", &de), (4, 6, "+ 2", &en)]
    {
        for w in [340.0, 1100.0] {
            let mut c = Card::limited("k", "K", Limit::new(limit).unwrap());
            for i in 0..facts {
                c = c.fact_or_more(FieldSlot::new(
                    "field-email",
                    "label.email",
                    &format!("{i}"),
                ));
            }
            let rig = rig(c, labels.clone(), w);
            let (_, card_right) = card_edges(&rig);
            let (left, right) = text_edges(&rig, word);
            let (lx, _, lw, _) = rig.bounds("B-card-k-line");
            assert!(
                card_right - right <= 24.0,
                "{word} at {w}: the note is {} px from the right edge",
                card_right - right
            );
            assert!(
                lx + lw <= left + 0.5,
                "{word} at {w}: the line reaches {}",
                lx + lw
            );
        }
    }
}

#[test]
fn no_note_within_the_limit() {
    for facts in [0, 3, 4] {
        let rig = rig(long_card(4, facts), Labels::default(), 1100.0);
        assert!(!rig.has("B-card-k-more"), "{facts} facts, limit 4");
    }
    let rig = rig(long_card(4, 5), Labels::default(), 1100.0);
    assert!(rig.has("B-card-k-more"));
}
