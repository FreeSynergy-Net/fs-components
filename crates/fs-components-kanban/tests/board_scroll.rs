//! A carry in a window smaller than the board and in a scrolled board: the places are measured from the layout, whatever its size and
//! scroll. Mutants: the scroll of the layout is not read from the tree -> the card lands elsewhere. Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Spot;
use fs_components_kanban::{BoardBench, Card, Column, Labels, MemoryBoard};

fn column(key: &str, cards: &[&str]) -> Column {
    let mut c = Column::new(key, key);
    c.cards = cards
        .iter()
        .map(|k| Card::new(k, &k.to_uppercase()))
        .collect();
    c
}

fn board() -> MemoryBoard {
    MemoryBoard::new(vec![
        column("todo", &["a", "b", "c"]),
        column("doing", &["d"]),
        column("done", &[]),
    ])
}

fn bench(board: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(board.clone()), Labels::default())
        .events(Arc::new(board.clone()))
        .id("B")
}

/// A window smaller than the board (the board scrolls; the places are read from the layout of that size, whatever it is): the carry
/// still finds its place. Zoom of the places themselves (0.5 to 2.0, scroll without layout) is `card_measured` in the widgets.
#[test]
fn a_carry_finds_its_place_in_a_window_smaller_than_the_board() {
    let b = board();
    let mut rig = board_common::Rig::new(bench(&b), 700.0, 420.0);
    board_common::carry_in(&mut rig, "B-card-a", "B-col-doing", Spot::End);
    assert_eq!(b.log(), ["moved:a:todo>doing@d"]);
}

/// The scroll of the last layout is READ from the tree (where the content lies against the scroll box), not guessed: with the board
/// scrolled, the card is put down where the pointer is. (Scroll WITHOUT a layout in between is `card_measured` of the widgets, which
/// tests `laid_out_at`; Blinc's wheel cannot be driven in the rig without a layout.)
#[test]
fn a_scrolled_board_puts_the_card_where_the_pointer_is() {
    let b = board();
    let mut rig = board_common::Rig::new(bench(&b), 700.0, 260.0);
    rig.bench.scroll_to_offset(0.0, 40.0);
    rig.frame();
    let (x, y, w, h) = rig.bounds("B-col-doing");
    assert!(y < 48.0 || h > 0.0);
    rig.drag_to("B-card-a", (x + w / 2.0, (y + 20.0).max(60.0)));
    assert_eq!(b.log(), ["moved:a:todo>doing@^"]);
}
