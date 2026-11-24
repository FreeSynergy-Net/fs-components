//! The form `bars-in-time` drawn: lanes with bars, states, dependency lines; two pictures (kept when `FS_BARS_PICTURES` names a folder).
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_styles::StyleId;
use fs_components_timeline::*;
use host::{H, W, bars_timeline, d, seed_records};
use ui::{background, draw_tree, ink, layout, serial};

const GW: f32 = 1000.0;
const GH: f32 = 560.0;

fn arrow_ink(records: Vec<BarRecord>) -> usize {
    let t = bars_timeline(StyleId::Admin, records);
    let tree = layout(t.build(GW, GH), GW, GH);
    let b = tree
        .get_absolute_bounds(tree.query_by_id("tl-arrows").unwrap())
        .unwrap();
    let img = draw_tree(&tree, GW as u32, GH as u32, "bars-arrows.png");
    ink(
        &img,
        (b.x as u32, b.y as u32, b.width as u32, b.height as u32),
        background(&img),
    )
}

/// Mutant guard: drawing WITHOUT the `depends` line (the same records, `depends` emptied) must show less ink in the arrow layer.
#[test]
fn the_depends_lines_are_drawn() {
    let _g = serial();
    let with = arrow_ink(seed_records());
    let mut bare = seed_records();
    bare.iter_mut().for_each(|r| r.depends.clear());
    let without = arrow_ink(bare);
    assert!(
        with > without + 60,
        "three arrows add ink: {with} vs {without}"
    );
}

/// The colour of the bar `key` where nothing is done yet (right end, inside the frame).
fn bar_pixel(records: Vec<BarRecord>, key: &str) -> [u8; 4] {
    let t = bars_timeline(StyleId::Admin, records);
    let tree = layout(t.build(GW, GH), GW, GH);
    let b = tree
        .get_absolute_bounds(tree.query_by_id(&format!("tl-item-{key}")).unwrap())
        .unwrap();
    let img = draw_tree(&tree, GW as u32, GH as u32, "bars-state.png");
    img.get_pixel((b.x + b.width - 4.0) as u32, (b.y + 3.0) as u32)
        .0
}

/// Mutant guard: a bar WITHOUT its state is drawn in another colour than the same bar with state `failed` / `cancelled`.
#[test]
fn the_state_changes_what_is_drawn() {
    let _g = serial();
    let rec = |s: Option<&str>| {
        let mut r = BarRecord::new("t", "T", d(2026, 11, 9), d(2026, 11, 20)).lane("L");
        r.state = s.map(Into::into);
        vec![r]
    };
    let plain = bar_pixel(rec(None), "t");
    let failed = bar_pixel(rec(Some("failed")), "t");
    let done = bar_pixel(rec(Some("completed")), "t");
    let cancelled = bar_pixel(rec(Some("cancelled")), "t");
    assert_ne!(plain, failed, "failed is not the plain colour");
    assert_ne!(failed, done, "failed and completed differ");
    assert!(
        failed[0] > failed[1],
        "failed leans to the error colour: {failed:?}"
    );
    assert!(
        done[1] > done[0],
        "completed leans to the success colour: {done:?}"
    );
    // cancelled is an outline: the inside is the page, not a tint
    let t = bars_timeline(StyleId::Admin, rec(Some("cancelled")));
    let tree = layout(t.build(GW, GH), GW, GH);
    let img = draw_tree(&tree, GW as u32, GH as u32, "bars-cancel.png");
    assert_eq!(
        cancelled,
        *background(&img).as_ref(),
        "outline: the inside shows the background"
    );
}

#[test]
fn lane_headings_and_bars_are_in_the_tree_and_bars_stand_below_their_heading() {
    let _g = serial();
    let t = bars_timeline(StyleId::Admin, seed_records());
    let tree = layout(t.build(GW, GH), GW, GH);
    let y = |id: &str| {
        tree.get_absolute_bounds(tree.query_by_id(id).unwrap_or_else(|| panic!("no {id}")))
            .unwrap()
            .y
    };
    for lane in ["Umzug Hafenstraße", "Website-Relaunch", "Alltag"] {
        assert!(
            tree.query_by_id(&format!("tl-lane-{lane}")).is_some(),
            "{lane}"
        );
    }
    assert!(y("tl-lane-Umzug Hafenstraße") < y("tl-item-mietvertrag"));
    assert!(y("tl-item-einpacken") < y("tl-lane-Website-Relaunch"));
    assert!(y("tl-item-farben") < y("tl-lane-Alltag"));
    assert!(y("tl-lane-Alltag") < y("tl-item-abo"));
}

#[test]
fn empty_and_single_bar_plans_draw() {
    let _g = serial();
    for records in [vec![], seed_records().into_iter().take(1).collect()] {
        let n = records.len();
        let t = bars_timeline(StyleId::Windows, records);
        let tree = layout(t.build(W, H), W, H);
        assert_eq!(tree.query_by_id("tl-item-mietvertrag").is_some(), n == 1);
        assert!(tree.query_by_id("tl-arrows").is_some());
        draw_tree(&tree, W as u32, H as u32, "bars-small.png");
    }
}

/// The two pictures of the order: the Gantt with three lanes and `depends` lines, and the empty / one-bar case. Written only
/// when `FS_BARS_PICTURES` names a folder (as `bilder/timeline`).
#[test]
fn pictures_of_the_gantt_and_the_small_cases() {
    let _g = serial();
    let Some(dir) = std::env::var_os("FS_BARS_PICTURES") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut gantt = bars_timeline(StyleId::Admin, seed_records());
    gantt = gantt.events(std::sync::Arc::new(NoEvents));
    let tree = layout(gantt.build(GW, GH), GW, GH);
    draw_tree(&tree, GW as u32, GH as u32, "x.png")
        .save(dir.join("gantt_three_lanes.png"))
        .unwrap();
    let (sw, sh) = (700.0_f32, 200.0_f32);
    let mut small = Vec::new();
    for (name, records) in [
        ("empty", vec![]),
        (
            "one_bar",
            seed_records()
                .into_iter()
                .skip(1)
                .take(1)
                .map(|mut r| {
                    r.depends.clear();
                    r
                })
                .collect(),
        ),
    ] {
        let t = bars_timeline(StyleId::Admin, records);
        let tree = layout(t.build(sw, sh), sw, sh);
        let img = draw_tree(&tree, sw as u32, sh as u32, "x.png");
        img.save(dir.join(format!("gantt_{name}.png"))).unwrap();
        small.push(name);
    }
    assert_eq!(small.len(), 2);
}
