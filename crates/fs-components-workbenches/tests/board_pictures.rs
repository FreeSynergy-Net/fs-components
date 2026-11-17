//! Pictures of the board workbench in every state (split from `board_view.rs`): standard, over the limit, carried, carried and blocked,
//! held by the keyboard, quick entry open, frame at the side, notice with undo, the program's question. Light by default, dark with
//! `FS_RENDER_SCHEME=dark`, kept with `FS_COMP_SHOTS=<dir>`; skipped with a message where there is no graphics adapter.
//!
//! Tests: pixel tests. Pattern: none.
#![allow(clippy::pedantic)]

mod board_common;

use std::path::PathBuf;
use std::sync::Arc;

use board_common::{column, sample, serial};
use fs_components_render::{Canvas, RenderError, draw_to_png};
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
