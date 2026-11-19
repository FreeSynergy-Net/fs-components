//! Pictures of the bundle 3 (date-time with offset, duration) in the Admin style, light and dark; kept when `FS_COMP_SHOTS` names a
//! directory.
//!
//! Tests: draws every scene (input with a value), asserts that something was drawn and that dark differs from light.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use common::{background, draw_tree, ink, layout_with, serial};
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look, Say};
use fs_components_styles::StyleId;

#[test]
fn every_scene_is_drawn_light_and_dark() {
    let _g = serial();
    for (name, kind, value) in [
        ("date-time", Kind::DateTime, "2026-11-10T14:30:00+01:00"),
        ("duration", Kind::Duration, "P1DT2H30M"),
    ] {
        let mut ground = Vec::new();
        for dark in [false, true] {
            common::set_dark(dark);
            let look = Look::new(StyleId::Admin, dark);
            let say: Arc<dyn Say> = Arc::new(KeysAsText);
            let input = FieldInput::new(kind, look, say).value(value);
            let ui = div()
                .w(640.0)
                .h(150.0)
                .bg(Look::color(look.palette().ground))
                .p(16.0)
                .child(input.build());
            let file = format!("f3-{name}-{}.png", if dark { "dark" } else { "light" });
            let sheet = fs_components_styles::styles::block_tokens::sheet(look.style);
            let tree = layout_with(ui, 640.0, 150.0, &sheet);
            let img = draw_tree(&tree, 640, 150, &file);
            let bg = background(&img);
            assert!(
                ink(&img, (0, 0, 640, 150), bg) > 300,
                "{name} dark={dark}: nearly empty"
            );
            ground.push(bg);
        }
        assert_ne!(ground[0], ground[1], "{name}: dark is not dark");
    }
    common::set_dark(false);
}
