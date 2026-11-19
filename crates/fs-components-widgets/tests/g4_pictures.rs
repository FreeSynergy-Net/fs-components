//! Pictures of the bundle G4 (slider, range, meter, progress, splitter) in a FreeSynergy style, light and dark, with the theme guard
//! of `styled.rs` ([THEME-WAECHTER]); kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene in the style Admin, light and dark, and asserts that something was drawn and that the dark picture
//! differs from the light one; the pictures are for a person to look at (rule: pictures before release).

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::meter::Meter;
use fs_components_widgets::progress::Progress;
use fs_components_widgets::range::Range;
use fs_components_widgets::slider::{Scale, Slider};
use fs_components_widgets::splitter::Splitter;
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 5] = [
    ("slider", 600, 120, slider),
    ("range", 640, 120, range),
    ("meter", 600, 200, meter),
    ("progress", 600, 200, progress),
    ("splitter", 600, 200, splitter),
];

fn slider() -> Div {
    let s = Slider::new(Scale::new(0.0, 100.0, 5.0), "Volume")
        .id("g4-sl")
        .value(65.0)
        .words(|v| format!("{v:.0} %"));
    div().w(500.0).child(s.build(360.0))
}

fn range() -> Div {
    let r = Range::new(Scale::new(0.0, 100.0, 5.0), "Lowest price", "Highest price")
        .id("g4-rg")
        .values(25.0, 70.0);
    div().w(560.0).child(r.build(340.0))
}

fn meter() -> Div {
    let scale = Scale::new(0.0, 100.0, 1.0);
    let row = |id: &str, name: &str, v: f64| {
        Meter::new(scale, name, v)
            .id(id)
            .zones(30.0, 70.0, 5.0)
            .build(300.0)
    };
    div()
        .w(500.0)
        .flex_col()
        .gap_px(10.0)
        .child(row("g4-m1", "Disk space", 18.0))
        .child(row("g4-m2", "Memory", 55.0))
        .child(row("g4-m3", "Battery use", 92.0))
}

fn progress() -> Div {
    let known = Progress::new("Copying").id("g4-p1");
    known.set(45.0);
    let done = Progress::new("Backup").id("g4-p2");
    done.set(100.0);
    let unknown = Progress::new("Scanning").id("g4-p3").unknown();
    div()
        .w(500.0)
        .flex_col()
        .gap_px(10.0)
        .child(known.build(300.0))
        .child(done.build(300.0))
        .child(unknown.build(300.0))
}

fn splitter() -> Div {
    let s = Splitter::new("Sidebar width", 35.0, 10.0, 80.0, 5.0).id("g4-sp");
    div().child(s.build(
        508.0,
        120.0,
        || div().p(10.0).child(text("Sidebar")),
        || div().p(10.0).child(text("Content")),
    ))
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("g4-{name}.png");
        let light = draw_styled(build, w, h, StyleId::Admin, false, &css, &file);
        let dark = draw_styled(build, w, h, StyleId::Admin, true, &css, &file);
        for (img, mode) in [(&light, "light"), (&dark, "dark")] {
            let ground = ground_of(img);
            let ink = img
                .enumerate_pixels()
                .filter(|(x, y, _)| *x >= STRIP + 4 || *y >= SWATCH + 4)
                .filter(|(_, _, p)| {
                    (0..3)
                        .map(|i| i32::from(p.0[i].abs_diff(ground[i])))
                        .sum::<i32>()
                        > 40
                })
                .count();
            assert!(ink > 300, "{name} {mode}: nearly empty ({ink} ink pixels)");
        }
        assert_ne!(
            ground_of(&light),
            ground_of(&dark),
            "{name}: dark is not dark"
        );
    }
}
