//! Pictures of the help in a FreeSynergy style, light and dark, with the theme guard of `styled.rs`; kept when `FS_COMP_SHOTS` names a
//! directory.
//!
//! Tests: draws every outcome (an entry, a translation that is old and not in the language, no entry, no source) in the style Admin,
//! light and dark; asserts that something was drawn and that the dark picture is dark.

mod common;
#[path = "common/styled.rs"]
mod styled;

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::help::{Help, HelpBlock, HelpError, HelpSource, Labels};
use styled::{STRIP, SWATCH, draw_styled, ground_of};

struct Fixed(Result<Option<HelpBlock>, HelpError>);

impl HelpSource for Fixed {
    fn block(&self, _: &str, _: &str, _: &[String]) -> Result<Option<HelpBlock>, HelpError> {
        self.0.clone()
    }
}

fn help(answer: Result<Option<HelpBlock>, HelpError>, id: &str) -> Div {
    let h = Help::new(&Labels::default(), "fs-files", Arc::new(Fixed(answer)))
        .id(id)
        .on_docs(|| {});
    div().w(520.0).child(h.build("versions"))
}

fn entry(old: bool) -> HelpBlock {
    HelpBlock {
        id: "b1".into(),
        lang: "en".into(),
        text: "A version is a state of a file.\n\nOpen the versions from the menu File, then choose one and press Restore.".into(),
        rev: 3,
        fallback_from: old.then(|| "de".to_owned()),
        outdated: old,
    }
}

fn found() -> Div {
    help(Ok(Some(entry(false))), "h1")
}
fn translated_old() -> Div {
    help(Ok(Some(entry(true))), "h2")
}
fn missing() -> Div {
    help(Ok(None), "h3")
}
fn unavailable() -> Div {
    help(Err(HelpError("no texts".into())), "h4")
}

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 4] = [
    ("found", 620, 220, found),
    ("old-translation", 620, 260, translated_old),
    ("missing", 620, 200, missing),
    ("unavailable", 620, 220, unavailable),
];

#[test]
fn every_outcome_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("help-{name}.png");
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
            assert!(ink > 600, "{name} {mode}: nearly empty ({ink} ink pixels)");
        }
        assert_ne!(
            ground_of(&light),
            ground_of(&dark),
            "{name}: dark is not dark"
        );
    }
}
