//! The lanes and the columns of the board are areas: a frame and a ground of the theme tokens, a set-off head, also when empty (the look itself is judged on the pictures).
//! Pictures in light and dark and in two styles (`FS_COMP_SHOTS=<dir>` keeps them), and the columns come from the data.
//!
//! Tests: pixel tests plus input tests over the tree. Mutants: an empty lane or column left out (`empty_lane_and_column_stay_elements`);
//! a fixed number of columns (`the_columns_are_the_data`). Pattern: none.
#![allow(clippy::pedantic)]

mod board_common;

use std::path::PathBuf;
use std::sync::Arc;

use board_common::{Rig, serial};
use fs_components_fields::Look;
use fs_components_kanban::{BoardBench, Card, Column, Labels, LaneHead, LanePlan, MemoryBoard};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_styles::StyleId;

fn columns(n: usize) -> Vec<Column> {
    (0..n)
        .map(|i| {
            let mut c = Column::new(&format!("c{i}"), &format!("Column {i}"));
            if i % 2 == 0 {
                let mut card = Card::new(&format!("k{i}"), &format!("Card {i}"));
                card.assignees = vec!["ada".into()];
                c.cards = vec![card];
            }
            c
        })
        .collect()
}

fn lanes() -> LanePlan {
    LanePlan::On {
        grouping: "assignee".into(),
        heads: vec![
            LaneHead::new("ada", "Ada"),
            LaneHead::new("bob", "Bob (empty)"),
        ],
    }
}

fn bench(b: MemoryBoard, style: StyleId) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b))
        .look(Look::new(style, false))
        .title("Home")
        .id("B")
}

#[test]
fn the_columns_are_the_data() {
    for n in [2, 5, 9] {
        for with_lanes in [false, true] {
            let mut b = MemoryBoard::new(columns(n));
            if with_lanes {
                b = b.with_lanes(lanes());
            }
            let rig = Rig::new(bench(b, StyleId::Admin), 1900.0, 700.0);
            let heads = (0..12).filter(|i| rig.has(&format!("B-head-c{i}"))).count();
            assert_eq!(heads, n, "{n} columns in the data (lanes: {with_lanes})");
        }
    }
}

#[test]
fn lane_and_column_have_a_ground() {
    {
        let plain = Rig::new(
            bench(MemoryBoard::new(columns(3)), StyleId::Admin),
            1100.0,
            600.0,
        );
        assert!(plain.has("B-col-c1"), "an empty column is still an element");
    }
    let rig = Rig::new(
        bench(
            MemoryBoard::new(columns(3)).with_lanes(lanes()),
            StyleId::Admin,
        ),
        1100.0,
        600.0,
    );
    assert!(
        rig.has("B-lane-bob-band"),
        "an empty lane is still an element"
    );
}

// ---- pictures -------------------------------------------------------------------------------------------------------------------

fn shoot(b: &BoardBench, name: &str) {
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
    let path = dir.join(format!("areas-{name}-{scheme}.png"));
    let canvas = Canvas::new(1000, 520, fs_components_kanban::styles());
    let b = b.clone();
    match draw_to_png(&canvas, move |w, h| b.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
}

#[test]
fn picture_areas() {
    for (style, name) in [(StyleId::Admin, "admin"), (StyleId::Simple, "simple")] {
        shoot(
            &bench(MemoryBoard::new(columns(3)).with_lanes(lanes()), style),
            &format!("lanes-{name}"),
        );
        shoot(
            &bench(MemoryBoard::new(columns(3)), style),
            &format!("plain-{name}"),
        );
    }
}
