//! Group `render`: what `fs-components-render` draws itself: the theme variables of the style sheet and a line of text
//! in the system's mono face (the one Craft falls back to; no font is bundled, ADR 0089).
//!
//! Tests: picture comparison against `contract/umbau/bilder/render` (the colour strips); the text line is read, not compared.

use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_fields::Look;
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("render", (200, 64), Tolerance::RASTER)
}

#[test]
fn the_style_sheet_reaches_the_picture() {
    let css = ".u-box { background: var(--primary); } .u-err { background: var(--error-bg); }";
    group().check("sheet_vars", css, |_, _| {
        div()
            .flex_row()
            .child(div().w(100.0).h(64.0).class("u-box"))
            .child(div().w(100.0).h(64.0).class("u-err"))
    });
}

#[test]
fn a_line_of_text_in_the_system_mono_face_is_drawn_in_light_text() {
    // No font is bundled (ADR 0089): Craft falls back to the system's `monospace` when `JetBrains Mono` is not
    // installed, and the metric differs from machine to machine. So there is no reference picture of the glyphs: the
    // picture is read for what holds on every machine: the ground stays dark and the text is light, left of the middle.
    if fs_components_render::installed_mono_face().is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "no monospace font installed on a CI machine: install one (for example fonts-dejavu-core)"
        );
        eprintln!(
            "SKIPPED: no monospace font installed (render/a_line_of_text_in_the_system_mono_face_is_drawn_in_light_text); this run is no proof"
        );
        return;
    }
    let drawn = group().draw("", |_, _| {
        let look = Look::new(StyleId::Craft, true); // light text: the picture ground is dark
        div().p(8.0).child(
            text("Craft 0123 fs")
                .size(14.0)
                .color(Look::color(look.palette().text))
                .monospace()
                .no_wrap(),
        )
    });
    let picture = match drawn {
        Ok(p) => p,
        Err(why) => {
            assert!(
                std::env::var_os("CI").is_none(),
                "cannot be drawn in CI: {why}"
            );
            eprintln!(
                "skipped render/a_line_of_text_in_the_system_mono_face_is_drawn_in_light_text: {why}"
            );
            return;
        }
    };
    let light = |x: u32, y: u32| picture.get_pixel(x, y).0[..3].iter().all(|c| *c > 150);
    let (w, h) = picture.dimensions();
    let in_text = (0..w / 2)
        .flat_map(|x| (0..h).map(move |y| (x, y)))
        .filter(|&(x, y)| light(x, y))
        .count();
    let in_far_corner = (w - 20..w)
        .flat_map(|x| (0..h).map(move |y| (x, y)))
        .filter(|&(x, y)| light(x, y))
        .count();
    assert!(in_text >= 40, "no light text drawn: {in_text} light pixels");
    assert_eq!(in_far_corner, 0, "the right edge must stay the dark ground");
}
