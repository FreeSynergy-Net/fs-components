//! The board element in process: the real layout, the real router, the pointer carrying a card, a click opening one, the foot of a
//! column, the frame at the side, the marks. Pictures of every state with `FS_COMP_SHOTS=<dir>` (light and, with
//! `FS_RENDER_SCHEME=dark`, dark); skipped with a message where there is no graphics adapter.
//!
//! Tests: input tests over Blinc's tree and router; pixel tests for the pictures. A hang here is a deadlock: run with a timeout.
//! Mutants: the board draws no mark (`the_marks_and_the_foot_are_there`); a card is not `CARD_H` high so the pointer tests lie
//! (`a_card_is_as_high_as_the_hit_test_thinks`); the press does not reach the state (`a_click_without_a_carry_opens_the_card`); the
//! release does not (`the_pointer_carries_a_card_to_another_column`); the frame ignores the program (`opening_a_card_shows_the_frame`).
//! Pattern: none; tests of one element.

#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_kanban::sizes::{CARD_GAP, COLUMN_W};
use fs_components_kanban::{BoardBench, DetailContent, DetailFrame, Labels, MemoryBoard};

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        Some(
            DetailFrame::new(format!("Card {card}"))
                .path("Home > To do")
                .property("Assignee", "Ada")
                .property("Due", "")
                .primary("done", "Done"),
        )
    }
}

fn bench(b: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b.clone()))
        .detail(Arc::new(Detail))
        .title("Home")
        .id("B")
}

const SIZE: (f32, f32) = (1100.0, 640.0);

#[test]
fn the_marks_and_the_foot_are_there() {
    let b = sample();
    let rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    assert!(rig.has("B-head-todo") && rig.has("B-head-doing") && rig.has("B-head-done"));
    assert!(rig.has("B-wip-todo") && rig.has("B-wip-doing"));
    assert!(rig.has("B-add-todo"), "a foot in every column");
    assert!(rig.has("B-empty-done"), "an empty column says so");
    assert!(rig.has("B-card-a") && rig.has("B-card-d"));
    assert!(!rig.has("B-detail"), "no frame until a card is open");
}

#[test]
fn a_card_is_a_column_wide_and_as_high_as_its_content() {
    let b = sample();
    let rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    let (_, y0, w, h) = rig.bounds("B-card-a");
    let (_, y1, _, h1) = rig.bounds("B-card-b");
    assert!((w - COLUMN_W).abs() < 1.5, "card width {w}");
    assert!(
        h > 20.0 && (h - h1).abs() < 1.5,
        "two one-line cards are alike: {h}, {h1}"
    );
    assert!(
        ((y1 - y0) - (h + CARD_GAP)).abs() < 1.5,
        "the step between cards {}",
        y1 - y0
    );
}

/// The cards stand inside their cell, in the grid, from the origin of the content (the board was once pushed to the right by a
/// full-width layer in a row). Mutant: the content is a row again -> the first cell is far to the right and this is red.
#[test]
fn the_cards_stand_inside_their_cells() {
    let b = sample();
    let rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    let (ox, oy, ..) = rig.bounds("B-content");
    for (key, column) in [("a", "todo"), ("b", "todo"), ("d", "doing")] {
        let (cx, cy, cw, ch) = rig.bounds(&format!("B-card-{key}"));
        let (x, y, w, h) = rig.bounds(&format!("B-col-{column}"));
        assert!(
            cx >= x && cx + cw <= x + w + 0.5,
            "{key}: x {cx} in {x}..{}",
            x + w
        );
        assert!(
            cy >= y && cy + ch <= y + h + 0.5,
            "{key}: y {cy} in {y}..{}",
            y + h
        );
    }
    let (fx, ..) = rig.bounds("B-col-todo");
    assert!(
        (fx - ox).abs() < 4.0,
        "the first cell is at the left edge of the content ({fx} against {ox}, {oy})"
    );
}

#[test]
fn a_click_without_a_carry_opens_the_card() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.click("B-card-b");
    assert_eq!(b.log(), ["opened:b"]);
    assert!(
        rig.bench
            .read(|s| s.open_card().map(str::to_owned))
            .as_deref()
            == Some("b")
    );
}

#[test]
fn opening_a_card_shows_the_frame() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open("a");
    rig.rebuild();
    assert!(rig.has("B-detail"));
    assert!(rig.has("B-detail-primary"));
    rig.click("B-detail-close");
    rig.rebuild();
    assert!(!rig.has("B-detail"));
}

#[test]
fn the_pointer_carries_a_card_to_another_column() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    let (x, y, w, _) = rig.bounds("B-col-done");
    rig.drag_to("B-card-a", (x + w / 2.0, y + 40.0));
    assert_eq!(b.log(), ["moved:a:todo>done@^"]);
    rig.rebuild();
    let (dx, ..) = rig.bounds("B-card-a");
    assert!(dx > x - 1.0, "the card now stands in the third column");
}

#[test]
fn a_small_wiggle_is_a_click_not_a_carry() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    let (x, y) = rig.centre("B-card-c");
    rig.down(x, y);
    rig.move_to(x + 2.0, y + 1.0);
    rig.up(x + 2.0, y + 1.0);
    rig.frame();
    assert_eq!(b.log(), ["opened:c"]);
}

#[test]
fn the_foot_opens_a_quick_entry_and_a_title_makes_a_card() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.click("B-add-todo");
    rig.rebuild();
    assert!(rig.has("B-quick-todo"), "the entry stands at the foot");
    assert!(!rig.has("B-add-todo"));
    rig.bench.quick_submit("todo", "Pay the rent", false);
    rig.rebuild();
    assert_eq!(b.log(), ["new:todo:Pay the rent:false"]);
    assert_eq!(
        b.snapshot()[0]
            .cards
            .last()
            .map(|c| c.title.clone())
            .as_deref(),
        Some("Pay the rent")
    );
}

#[test]
fn several_lines_ask_first() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open_quick("todo");
    rig.bench.quick_submit("todo", "one\ntwo\nthree", false);
    rig.rebuild();
    assert!(rig.has("B-many"), "Create 3 cards?");
    assert!(b.log().is_empty());
    rig.bench.answer_many(true);
    assert_eq!(b.log().len(), 3);
    assert!(rig.bench.pending_many().is_none());
}

#[test]
fn the_keyboard_reaches_the_board() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    assert!(rig.focus_board("B"), "the board is a tab stop");
    rig.key(39, true); // Shift+Right
    assert_eq!(b.log(), ["moved:a:todo>doing@^"]);
}
