//! Pictures of the bundle G1 (tabs, toolbar, disclosure, accordion, breadcrumb) in a FreeSynergy style, light and dark, with the theme guard
//! of `styled.rs` ([THEME-WAECHTER]); kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene in the style Admin, light and dark, and asserts that something was drawn and that the dark picture
//! differs from the light one; the pictures are for a person to look at (rule: pictures before release).

mod common;
#[path = "common/styled.rs"]
mod styled;

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb};
use fs_components_widgets::disclosure::{Accordion, Disclosure, Section};
use fs_components_widgets::tabs::{Tab, Tabs};
use fs_components_widgets::toolbar::{Item, Toolbar};
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 6] = [
    ("tabs", 600, 240, tabs),
    ("tabs-manual", 600, 240, tabs_manual),
    ("toolbar", 600, 140, toolbar),
    ("disclosure", 600, 200, disclosure),
    ("accordion", 600, 300, accordion),
    ("breadcrumb", 600, 100, breadcrumb),
];

fn tab_row(id: &str) -> Tabs {
    Tabs::new(vec![
        Tab::new("one", "Overview"),
        Tab::new("two", "Sharing").disabled(),
        Tab::new("three", "History"),
        Tab::new("four", "Settings"),
    ])
    .id(id)
    .name("Sections")
}

fn tabs() -> Div {
    let t = tab_row("g1-tabs");
    t.choose("three");
    div().w(520.0).flex_col().child(t.build()).child(t.panel(|key| div().child(text(format!("The {key} panel")))))
}

fn tabs_manual() -> Div {
    // The cursor is on "Settings" while "Overview" is chosen: a raw key moves it without choosing.
    let t = tab_row("g1-tabsm").manual();
    t.press(fs_components_widgets::Key::Right);
    t.press(fs_components_widgets::Key::Right);
    div().w(520.0).flex_col().child(t.build()).child(t.panel(|key| div().child(text(format!("The {key} panel")))))
}

fn toolbar() -> Div {
    let t = Toolbar::new(vec![
        Item::button("cut", "Cut"),
        Item::button("copy", "Copy").disabled(),
        Item::separator(),
        Item::toggle("bold", "Bold", true),
        Item::toggle("italic", "Italic", false),
    ])
    .id("g1-tl")
    .name("Edit");
    div().w(520.0).child(t.build())
}

fn disclosure() -> Div {
    let d = Disclosure::new("Details").id("g1-dc").opened();
    div().w(520.0).child(d.build(|| div().child(text("Created by Ada, last changed today."))))
}

fn accordion() -> Div {
    let a = Accordion::new(vec![
        Section::new("a", "General", || div().child(text("Name and language."))),
        Section::new("b", "Sharing", || div().child(text("Who can read this."))),
        Section::new("c", "History", || div().child(text("Earlier versions."))),
    ])
    .id("g1-ac")
    .single();
    a.set_open("b", true);
    div().w(520.0).child(a.build())
}

fn breadcrumb() -> Div {
    let steps = vec![
        Crumb::new("home", "Home"),
        Crumb::new("docs", "Documents"),
        Crumb::new("year", "2026"),
        Crumb::new("here", "Report"),
    ];
    div().w(520.0).child(Breadcrumb::new(steps, "g1-bc").on_choose(Arc::new(|_| {})).build(520.0))
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("g1-{name}.png");
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
