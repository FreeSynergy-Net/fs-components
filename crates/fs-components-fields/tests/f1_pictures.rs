//! Pictures of the bundle 1 fields in the Admin style, light and dark; kept when `FS_COMP_SHOTS` names a directory
//! (`bilder/f1a-1`: link display, many-line input; `bilder/f1b-1`: switch, number).
//!
//! Tests: draws every scene, asserts that something was drawn and that the dark picture differs from the light one.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use common::{background, draw_tree, ink, layout_with, serial};
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look, Say, display_with};
use fs_components_styles::StyleId;

fn on_ground<I: blinc_layout::prelude::ElementBuilder + 'static>(
    look: &Look,
    w: f32,
    h: f32,
    ui: I,
) -> impl blinc_layout::prelude::ElementBuilder + use<I> {
    div()
        .w(w)
        .h(h)
        .bg(Look::color(look.palette().ground))
        .p(16.0)
        .child(ui)
}

#[test]
fn every_scene_is_drawn_light_and_dark() {
    let _g = serial();
    for (name, w, h) in [
        ("link", 460u32, 120u32),
        ("multiline", 460, 190),
    ] {
        let mut ground = Vec::new();
        for dark in [false, true] {
            common::set_dark(dark);
            let look = Look::new(StyleId::Admin, dark);
            let say: Arc<dyn Say> = Arc::new(KeysAsText);
            let file = format!(
                "{}-{name}-{}.png",
                "f1a",
                if dark { "dark" } else { "light" }
            );
            let sheet = format!(
                "{}\n{}\n{}",
                fs_components_fields::controls::STYLES,
                fs_components_fields::area_styles(),
                fs_components_styles::styles::block_tokens::sheet(look.style)
            );
            let ui = match name {
                "link" => div().child(display_with(
                    Kind::Link,
                    &["https://example.org/page".to_owned()],
                    &look,
                    &*say,
                    &None,
                )),
                _ => div().child(
                    FieldInput::new(Kind::Multiline, look, say)
                        .name("pic-ml")
                        .value("first line\nsecond line\nthird line")
                        .build(),
                ),
            };
            let tree = layout_with(
                on_ground(&look, w as f32, h as f32, ui),
                w as f32,
                h as f32,
                &sheet,
            );
            // `RenderTree::set_stylesheet` (the harness) does not set the sheet a text area reads; a window does (`set_stylesheet_arc`).
            blinc_layout::css_parser::set_active_stylesheet(Arc::new(
                blinc_layout::css_parser::Stylesheet::parse(&sheet).expect("sheet"),
            ));
            let img = draw_tree(&tree, w, h, &file);
            let bg = background(&img);
            assert!(
                ink(&img, (0, 0, w, h), bg) > 200,
                "{name} dark={dark}: nearly empty"
            );
            ground.push(bg);
        }
        assert_ne!(ground[0], ground[1], "{name}: dark is not dark");
    }
    common::set_dark(false);
}
