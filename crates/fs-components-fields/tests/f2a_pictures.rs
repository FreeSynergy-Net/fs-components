//! Pictures of the bundle 2a (choice as segments and as a list, keywords with chips) in the Admin style, light and dark; kept when
//! `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene, asserts that something was drawn and that the dark picture differs from the light one.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use common::{background, draw_tree, ink, layout_with, serial};
use fs_components_fields::{
    Candidate, ChoiceInput, KeysAsText, KeywordsInput, Look, ReferenceInput, ReferenceSource,
};
use fs_components_styles::StyleId;

struct Known;

impl ReferenceSource for Known {
    fn find(&self, _typed: &str, _limit: usize) -> Vec<Candidate> {
        vec![
            Candidate::new("https://example.org/tasks/7", "Write the report"),
            Candidate::new("https://example.org/tasks/8", "Write the plan"),
        ]
    }
}

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
        ("choice-segments", 500u32, 130u32),
        ("choice-list", 500, 130),
        ("keywords", 500, 170),
        ("reference", 500, 220),
        ("checklist", 500, 220),
    ] {
        let mut ground = Vec::new();
        for dark in [false, true] {
            common::set_dark(dark);
            let look = Look::new(StyleId::Admin, dark);
            let say: Arc<dyn fs_components_fields::Say> = Arc::new(KeysAsText);
            let ui = match name {
                "choice-segments" => div().child(
                    ChoiceInput::new(
                        look,
                        say,
                        "color",
                        vec!["red".into(), "green".into(), "blue".into()],
                    )
                    .name("p-ch")
                    .value("green")
                    .build(),
                ),
                "choice-list" => div().child(
                    ChoiceInput::new(
                        look,
                        say,
                        "color",
                        (0..8).map(|i| format!("entry-{i}")).collect(),
                    )
                    .name("p-cl")
                    .value("entry-3")
                    .build(),
                ),
                "reference" => div().child(
                    ReferenceInput::new(look, say, "p-rf", Some(Arc::new(Known)))
                        .value("tasks")
                        .build(),
                ),
                "checklist" => {
                    use fs_components_widgets::checklist::{Checklist, Item, Labels};
                    div().child(
                        Checklist::new(
                            "Packing",
                            &Labels::default(),
                            vec![
                                Item::new("a", "Tent"),
                                Item::new("b", "Stove").done(),
                                Item::new("c", "Rope"),
                            ],
                        )
                        .id("p-cl")
                        .build(),
                    )
                }
                _ => div().child(
                    KeywordsInput::named(look, say, "p-kw")
                        .value("family, work, friends")
                        .build(),
                ),
            };
            let file = format!("f2a-{name}-{}.png", if dark { "dark" } else { "light" });
            let sheet = format!(
                "{}\n{}\n{}",
                fs_components_fields::controls::STYLES,
                fs_components_widgets::styles(),
                fs_components_styles::styles::block_tokens::sheet(look.style)
            );
            let tree = layout_with(
                on_ground(&look, w as f32, h as f32, ui),
                w as f32,
                h as f32,
                &sheet,
            );
            let img = draw_tree(&tree, w, h, &file);
            let bg = background(&img);
            assert!(
                ink(&img, (0, 0, w, h), bg) > 300,
                "{name} dark={dark}: nearly empty"
            );
            ground.push(bg);
        }
        assert_ne!(ground[0], ground[1], "{name}: dark is not dark");
    }
    common::set_dark(false);
}
