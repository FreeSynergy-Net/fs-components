//! Picture of a card with long facts and several assignees (the facts end in an ellipsis, the assignees are drawn). Light by default,
//! dark with `FS_RENDER_SCHEME=dark`, kept with `FS_COMP_SHOTS=<dir>`; skipped with a message where there is no graphics adapter.
//!
//! Tests: pixel tests. Pattern: none.
#![allow(clippy::pedantic)]

mod board_common;

use std::path::PathBuf;
use std::sync::Arc;

use board_common::serial;
use fs_components_kanban::{BoardBench, Card, Column, FieldSlot, Labels, MemoryBoard};
use fs_components_render::{Canvas, RenderError, draw_to_png};

fn board() -> MemoryBoard {
    let with = |key: &str, title: &str, facts: &[&str], who: &[&str]| {
        let mut c = Card::new(key, title);
        for f in facts {
            c = c.fact_or_more(FieldSlot::new("field-email", "label.email", f));
        }
        c.assignees = who.iter().map(|s| (*s).to_owned()).collect();
        c
    };
    let mut col = Column::new("todo", "To do");
    col.cards = vec![
        with(
            "a",
            "Long facts, many people",
            &[
                "ada.lovelace@example.org",
                "bob.the.builder@example.org",
                "carol@example.org",
            ],
            &[
                "Ada Lovelace",
                "Bob Builder",
                "Carol King",
                "Dave Grohl",
                "Erin Brockovich",
            ],
        ),
        with(
            "b",
            "Short facts",
            &["a@b.org", "c@d.org"],
            &["Ada Lovelace"],
        ),
    ];
    MemoryBoard::new(vec![col])
}

#[test]
fn picture_facts_fit() {
    let _g = serial();
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-board-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let scheme = if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    };
    let path = dir.join(format!("card-facts-fit-{scheme}.png"));
    let bench = BoardBench::new(Arc::new(board()), Labels::default())
        .title("Home")
        .id("B");
    let canvas = Canvas::new(480, 300, fs_components_kanban::styles());
    match draw_to_png(&canvas, move |w, h| bench.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
}
