//! A reason without a sentence is said by its code wherever the board shows one: the notice of a refused move and of a lock, the
//! reason at the ghost of a card carried to a column that refuses, the notice under the quick entry. Never an empty line.
//!
//! Mutants (each makes the named test fail): `reason.said()` -> `reason.text` in `state_moves.rs` (`refuse`, `Answer::Refused`) ->
//! `a_refusal_without_a_sentence_says_its_code` and `a_refused_move_without_a_sentence_says_its_code`; in `view_drag.rs` ->
//! `the_carried_card_blocked_without_a_sentence_says_the_code`; in `bench_foot.rs` -> `the_quick_entry_refused_without_a_sentence_says_the_code`.
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_kanban::{Answer, BoardBench, BoardEvents, Labels, MemoryBoard, NewCard, Reason};

fn bench(b: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b.clone()))
        .id("B")
}

fn carry(bench: &BoardBench, to: (f32, f32)) {
    let (x, y, ..) = bench.read(|s| s.geometry().card_rect("a")).unwrap();
    bench.press("a", (0.0, 0.0));
    bench.drag("a", (60.0, 60.0), (to.0 - x, to.1 - y));
}

fn notice(bench: &BoardBench) -> Option<(String, bool)> {
    bench.read(|s| s.notice().map(|n| (n.text.clone(), n.refused)))
}

#[test]
fn a_refusal_without_a_sentence_says_its_code() {
    let b = sample().refusing("done", Reason::new("no-right", ""));
    let bench = bench(&b);
    carry(&bench, (16.0 + 608.0 + 40.0, 100.0));
    bench.release();
    assert_eq!(notice(&bench), Some(("no-right".into(), true)));
}

#[test]
fn a_refused_move_without_a_sentence_says_its_code() {
    let b = sample().refusing_moves(Reason::new("changed", ""));
    let bench = bench(&b);
    carry(&bench, (16.0 + 304.0 + 40.0, 300.0));
    bench.release();
    assert_eq!(b.log().len(), 1, "the program was asked");
    assert_eq!(notice(&bench), Some(("changed".into(), true)));
}

#[test]
fn the_carried_card_blocked_without_a_sentence_says_the_code() {
    let b = sample().refusing("done", Reason::new("no-right", ""));
    let bench = bench(&b);
    carry(&bench, (16.0 + 608.0 + 40.0, 150.0));
    let rig = Rig::new(bench, 1100.0, 640.0);
    assert!(rig.has("B-reason"), "the blocked ghost gives its reason");
    let texts: Vec<String> = rig
        .tree
        .iter_nodes()
        .filter_map(|(_, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect();
    assert!(texts.iter().any(|t| t == "no-right"), "{texts:?}");
}

struct Refuses;

impl BoardEvents for Refuses {
    fn new_card(&self, _card: &NewCard) -> Answer {
        Answer::Refused(Reason::new("no-room", ""))
    }
}

#[test]
fn the_quick_entry_refused_without_a_sentence_says_the_code() {
    let b = sample();
    let bench = BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(Refuses))
        .id("B");
    bench.open_quick("todo");
    bench.quick_submit("todo", "Pay the rent", false);
    let mut rig = Rig::new(bench, 1100.0, 640.0);
    rig.rebuild();
    let texts: Vec<String> = rig
        .tree
        .iter_nodes()
        .filter_map(|(_, n)| match &n.element_type {
            blinc_layout::renderer::ElementType::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect();
    assert!(texts.iter().any(|t| t == "no-room"), "{texts:?}");
}
