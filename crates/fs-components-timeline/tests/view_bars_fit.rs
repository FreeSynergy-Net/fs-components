//! The Gantt fits its view: the axis and the grid run to the right end of the view, and a title that does not fit in its bar
//! stands right of the bar (behind the first stretch of its dependency arrow). Also the words: German texts instead of keys.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_styles::StyleId;
use fs_components_timeline::route::GAP;
use fs_components_timeline::*;
use host::{bars_timeline, d, seed_records};
use ui::{draw_tree, extent_x, layout, serial};

const GW: f32 = 1000.0;
const GH: f32 = 560.0;

fn bounds(tree: &blinc_layout::prelude::RenderTree, id: &str) -> (f32, f32, f32, f32) {
    let b = tree
        .get_absolute_bounds(tree.query_by_id(id).unwrap_or_else(|| panic!("no {id}")))
        .unwrap();
    (b.x, b.y, b.width, b.height)
}

/// Layout rule: the axis reaches the edge asked for, and not further than needed.
#[test]
fn extend_to_reaches_the_edge() {
    let scale = layout::Scale {
        origin: d(2026, 11, 1),
        px: 12.0,
    };
    let last = d(2026, 11, 30);
    assert_eq!(
        layout::extend_to(last, &scale, 100.0),
        last,
        "already there"
    );
    let wide = layout::extend_to(last, &scale, 1000.0);
    assert!(scale.x_of(wide.add_days(1)) >= 1000.0, "reaches the edge");
    assert!(scale.x_of(wide) < 1000.0, "and does not run a day further");
}

/// The grid and the axis labels reach the right end of the view (they ended at about half the width, where the last item ended).
#[test]
fn the_axis_and_the_grid_run_to_the_end_of_the_view() {
    let _g = serial();
    let t = bars_timeline(StyleId::Admin, seed_records());
    let tree = layout(t.build(GW, GH), GW, GH);
    let (_, y, w, h) = bounds(&tree, "tl-axis");
    assert!(w >= GW, "the axis is as wide as the view: {w}");
    let img = draw_tree(&tree, GW as u32, GH as u32, "bars-fit-axis.png");
    // the tick lines go down the rows: some ink in the last 100 px of the header rows, and in the rows below
    let (_, right) = extent_x(&img, y as u32, (y + 40.0) as u32).expect("ink in the header rows");
    assert!(right as f32 >= GW - 100.0, "grid ends at {right} of {GW}");
    let (_, right) =
        extent_x(&img, (y + h - 60.0) as u32, (y + h - 20.0) as u32).expect("ink below");
    assert!(
        right as f32 >= GW - 100.0,
        "grid in the rows ends at {right}"
    );
}

/// A title that does not fit in its bar stands right of it, behind the first stretch of the arrow; one that fits stays inside.
#[test]
fn a_title_that_does_not_fit_stands_right_of_its_bar() {
    let _g = serial();
    let t = bars_timeline(StyleId::Admin, seed_records());
    let tree = layout(t.build(GW, GH), GW, GH);
    let body = bounds(&tree, "tl-body");
    for key in [
        "mietvertrag",
        "kartons",
        "einpacken",
        "navigation",
        "foto",
        "farben",
        "abo",
    ] {
        let bar = bounds(&tree, &format!("tl-item-{key}"));
        let title = bounds(&tree, &format!("tl-title-{key}"));
        assert!(
            title.0 >= bar.0 + bar.2 + GAP,
            "{key}: title at {} must be behind the bar end {} and the arrow stretch",
            title.0,
            bar.0 + bar.2
        );
        assert!(
            title.0 + title.2 <= body.0 + body.2,
            "{key}: not cut at the end of the body"
        );
        assert!(
            (title.1 - bar.1).abs() < 1.0,
            "{key}: on the row of its bar"
        );
    }
    for key in ["strom", "startseite"] {
        assert!(
            tree.query_by_id(&format!("tl-title-{key}")).is_none(),
            "{key} fits in its bar: no title beside it"
        );
    }
}

/// The pictures of this order, written only when `FS_BARS_PICTURES` names a folder: the Gantt with texts in German (the today
/// line stands on the fixed test day 16 Nov 2026, the name says so).
#[test]
fn picture_of_the_gantt_with_german_texts() {
    let _g = serial();
    let Some(dir) = std::env::var_os("FS_BARS_PICTURES") else {
        return;
    };
    let de = fs_components_words::Locale::parse("de").unwrap();
    let t = bars_timeline(StyleId::Admin, seed_records())
        .localised(&de, Arc::new(fs_components_mf2::NeutralDateNames));
    let tree = layout(t.build(GW, GH), GW, GH);
    draw_tree(&tree, GW as u32, GH as u32, "x.png")
        .save(std::path::Path::new(&dir).join("gantt_three_lanes_de_heute-2026-11-16.png"))
        .unwrap();
    let _ = (NoEvents, d(2026, 1, 1));
}
