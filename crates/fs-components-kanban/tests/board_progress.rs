//! SW-2, moved to the one board element (from `view_progress.rs`): the progress bar of a card (optional channel `progress` of
//! `cards-in-columns@1.1.0`): drawn only with a value, its filled part is the percentage of the bar, a value over 100 is cut to 100.
//! The card keeps its height (`CARD_H`) with a bar.
//!
//! Mutants (each makes the named test fail): the bar is drawn for every card -> `only_a_card_with_progress_has_a_bar`; the fill
//! ignores the value (always full) -> `the_fill_is_the_percentage`; the bar makes the card higher -> `a_bar_does_not_change_the_height_of_the_card`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_kanban::board::hit::CARD_H;
use fs_components_kanban::board::{BoardBench, Card, Column, Labels, MemoryBoard};

fn rig(progress: Option<u8>) -> Rig {
    let mut card = Card::new("a", "Write the spec");
    card.progress = progress;
    let mut col = Column::new("todo", "To do");
    col.cards = vec![card];
    let bench = BoardBench::new(Arc::new(MemoryBoard::new(vec![col])), Labels::default()).id("B");
    Rig::new(bench, 1100.0, 640.0)
}

fn bar_width(progress: Option<u8>) -> Option<(f32, f32)> {
    let rig = rig(progress);
    if !rig.has("B-card-a-progress") {
        return None;
    }
    let (_, _, full, _) = rig.bounds("B-card-a-progress");
    let (_, _, fill, _) = rig.bounds("B-card-a-progress-fill");
    Some((full, fill))
}

#[test]
fn only_a_card_with_progress_has_a_bar() {
    assert!(bar_width(None).is_none());
    assert!(bar_width(Some(0)).is_some());
}

#[test]
fn the_fill_is_the_percentage() {
    let (full, fill) = bar_width(Some(25)).unwrap();
    assert!(full > 100.0, "the bar has the width of the card ({full})");
    assert!((fill - full / 4.0).abs() < 1.0, "{fill} of {full}");
    let (full, fill) = bar_width(Some(250)).unwrap();
    assert!((fill - full).abs() < 1.0, "cut to 100: {fill} of {full}");
}

#[test]
fn a_bar_does_not_change_the_height_of_the_card() {
    for progress in [None, Some(40)] {
        let (_, _, _, h) = rig(progress).bounds("B-card-a");
        assert!((h - CARD_H).abs() < 1.5, "{progress:?}: card height {h}");
    }
}
