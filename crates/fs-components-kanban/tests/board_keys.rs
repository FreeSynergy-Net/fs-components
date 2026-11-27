//! The keyboard of the board is the keyboard of the grid: ONE tab stop; outside a cell the arrows walk from cell to cell, Enter goes in
//! (the cursor of the cards stands on the first card of the cell), inside the keys are the board's (walk over the cards, hold, Shift+arrow),
//! Escape comes back out (`Grid::leave_cell`). Escape over a place that refuses leaves no notice.
//!
//! Tests: input tests over the real layout. Mutants: the board root or a card is a tab stop too (`one_tab_stop_the_grid`); the arrows
//! outside a cell move a card (`outside_a_cell_the_arrows_walk_the_cells`); Escape does not come out (`escape_comes_back_out_of_the_cell`);
//! Escape counts as a drop (`escape_over_a_refusing_place_leaves_no_notice`). Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_kanban::{BoardBench, Labels, MemoryBoard, Reason};

fn bench(b: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b.clone()))
        .id("B")
}

#[test]
fn one_tab_stop_the_grid() {
    let mut rig = Rig::new(bench(&sample()), 1100.0, 700.0);
    let seen = rig.tab_stops(&["B", "B-grid", "B-card-a", "B-card-b", "B-card-d"]);
    assert_eq!(seen, ["B-grid"], "only the grid takes the keyboard");
}

#[test]
fn outside_a_cell_the_arrows_walk_the_cells() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), 1100.0, 700.0);
    assert!(rig.focus_board("B-grid"));
    rig.key(39, false); // right: the cell of "doing"
    rig.key(13, false); // Enter: into it
    assert_eq!(
        rig.bench.read(|s| s.cursor().map(str::to_owned)).as_deref(),
        Some("d")
    );
    assert!(b.log().is_empty(), "no card moved");
}

#[test]
fn inside_a_cell_the_board_keys_work_and_escape_comes_back_out() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), 1100.0, 700.0);
    assert!(rig.focus_board("B-grid"));
    rig.key(13, false);
    rig.key(40, false); // down: the next card
    assert_eq!(
        rig.bench.read(|s| s.cursor().map(str::to_owned)).as_deref(),
        Some("b")
    );
    rig.key(27, false); // Escape: out of the cell
    rig.key(39, false); // right: now the grid walks (the cursor of the cards stays)
    assert_eq!(
        rig.bench.read(|s| s.cursor().map(str::to_owned)).as_deref(),
        Some("b")
    );
    rig.key(13, false);
    assert_eq!(
        rig.bench.read(|s| s.cursor().map(str::to_owned)).as_deref(),
        Some("d"),
        "the cell of doing"
    );
}

#[test]
fn escape_over_a_refusing_place_leaves_no_notice() {
    let b = sample().refusing("done", Reason::new("no-right", "You may not finish cards."));
    let bench = bench(&b);
    board_common::hold(&bench, "a", "B-col-done", board_common::Spot::End);
    let mut rig = Rig::new(bench.clone(), 1500.0, 900.0);
    assert!(rig.focus_board("B-grid"));
    rig.key(27, false); // Escape: the carry is given up
    assert_eq!(bench.read(|s| s.notice().map(|n| n.text.clone())), None);
    assert!(!rig.has("B-ghost"), "the carry is over");
    assert!(b.log().is_empty());
}
