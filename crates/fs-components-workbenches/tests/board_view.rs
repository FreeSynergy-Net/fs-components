//! The board element in process: the real layout, the real router, the pointer carrying a card, a click opening one, the foot of a
//! column, the frame at the side, the marks. Pictures of every state with `FS_COMP_SHOTS=<dir>` (light and, with
//! `FS_RENDER_SCHEME=dark`, dark); skipped with a message where there is no graphics adapter.
//!
//! Tests: input tests over Blinc's tree and router; pixel tests for the pictures. A hang here is a deadlock: run with a timeout.
//! Mutants: the board draws no mark (`the_marks_and_the_foot_are_there`); a card is not `CARD_H` high so the pointer tests lie
//! (`a_card_is_as_high_as_the_hit_test_thinks`); the press does not reach the state (`a_click_without_a_carry_opens_the_card`); the
//! release does not (`the_pointer_carries_a_card_to_another_column`); the frame ignores the program (`opening_a_card_shows_the_frame`).
//! Pattern: none; tests of one element.

mod board_common;

use std::path::PathBuf;
use std::sync::Arc;

use board_common::{Rig, column, sample, serial};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::board::hit::{CARD_H, COLUMN_W};
use fs_components_workbenches::board::{
    BoardBench, DetailContent, DetailFrame, Labels, MemoryBoard,
};

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
fn a_card_is_as_high_as_the_hit_test_thinks() {
    let b = sample();
    let rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    let (_, y0, w, h) = rig.bounds("B-card-a");
    let (_, y1, ..) = rig.bounds("B-card-b");
    assert!((h - CARD_H).abs() < 1.5, "card height {h}");
    assert!((w - COLUMN_W).abs() < 1.5, "card width {w}");
    assert!(
        ((y1 - y0) - (CARD_H + 10.0)).abs() < 1.5,
        "the step between cards {}",
        y1 - y0
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
    let (x, y, w, _) = rig.bounds("B-head-done");
    rig.drag_to("B-card-a", (x + w / 2.0, y + 120.0));
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

// ---- pictures -------------------------------------------------------------------------------------------------------------------

fn scheme() -> &'static str {
    if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    }
}

fn shoot(bench: &BoardBench, name: &str) {
    let _g = serial();
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-board-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("board-{name}-{}.png", scheme()));
    let canvas = Canvas::new(
        SIZE.0 as u32,
        SIZE.1 as u32,
        fs_components_workbenches::board::styles(),
    );
    let bench = bench.clone();
    match draw_to_png(&canvas, move |w, h| bench.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            let colours: std::collections::HashSet<_> = picture.pixels().map(|p| p.0).collect();
            assert!(colours.len() > 20, "something was drawn");
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
}

#[test]
fn picture_standard() {
    shoot(&bench(&sample()), "standard");
}

#[test]
fn picture_wip_over() {
    // doing has a limit of 1 and holds two cards
    let b = MemoryBoard::new(vec![
        column("todo", "To do", &[("a", "Write the letter")]),
        {
            let mut c = column(
                "doing",
                "Doing",
                &[("d", "Call the landlord"), ("e", "Fix the tap")],
            );
            c.limit = Some(1);
            c
        },
        column("done", "Done", &[]),
    ])
    .with_hidden("todo", 2);
    shoot(&bench(&b), "wip-over");
}

#[test]
fn picture_carried_and_blocked() {
    let b = sample().refusing(
        "done",
        fs_components_workbenches::board::Reason::new("no-right", "You may not finish cards."),
    );
    let bench = bench(&b);
    bench.press("a", (0.0, 0.0));
    let (x, y, ..) = bench.read(|s| s.geometry().card_rect("a")).unwrap();
    bench.drag("a", (80.0, 80.0), (16.0 + 608.0 + 40.0 - x, 150.0 - y));
    shoot(&bench, "ziehen-gesperrt");
}

#[test]
fn picture_carried_in_the_middle() {
    let b = sample();
    let bench = bench(&b);
    bench.press("a", (0.0, 0.0));
    let (x, y, ..) = bench.read(|s| s.geometry().card_rect("a")).unwrap();
    bench.drag("a", (80.0, 80.0), (16.0 + 304.0 + 40.0 - x, 120.0 - y));
    shoot(&bench, "ziehen-mitte");
}

#[test]
fn picture_keyboard_holds() {
    let b = sample();
    let bench = bench(&b);
    bench.press_key(&fs_components_widgets::keys::KeyEvent::new(
        fs_components_widgets::keys::Key::Space,
    ));
    shoot(&bench, "tastatur-greifen");
}

#[test]
fn picture_quick_entry_open() {
    let b = sample();
    let bench = bench(&b);
    bench.open_quick("todo");
    shoot(&bench, "schnelleingabe-offen");
}

#[test]
fn picture_detail_frame() {
    let b = sample();
    let bench = bench(&b);
    bench.open("b");
    shoot(&bench, "details-seitenfeld");
}

#[test]
fn picture_notice_with_undo() {
    let b = sample();
    let bench = bench(&b);
    bench.run("board.to:done", "a");
    shoot(&bench, "rueckgaengig-meldung");
}

#[test]
fn picture_question() {
    let b = sample().asking_about("a");
    let bench = bench(&b);
    bench.run("board.to:done", "a");
    shoot(&bench, "frage-claim");
}
