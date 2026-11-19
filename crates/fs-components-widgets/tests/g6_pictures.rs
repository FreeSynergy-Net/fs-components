//! Pictures of the bundle G6 (carousel, chip) in a FreeSynergy style, light and dark, with the theme guard
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
use fs_components_widgets::carousel::{Carousel, Labels};
use fs_components_widgets::chip::{Chip, ChipList, Labels as ChipLabels};
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 2] = [("carousel", 600, 220, carousel), ("chips", 600, 120, chips)];

fn carousel() -> Div {
    let c = Carousel::new("Featured", &Labels::default())
        .id("g6-cr")
        .slide(|| div().child(text("First program")))
        .slide(|| div().child(text("Second program")))
        .slide(|| div().child(text("Third program")));
    c.go(1);
    div().w(500.0).child(c.build())
}

fn chips() -> Div {
    let c = ChipList::new(
        "Keywords",
        &ChipLabels::default(),
        vec![
            Chip::new("a", "alpha"),
            Chip::new("b", "beta"),
            Chip::new("c", "gamma"),
        ],
    )
    .id("g6-ch");
    c.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Right,
    ));
    div().w(500.0).child(c.build())
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("g6-{name}.png");
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
