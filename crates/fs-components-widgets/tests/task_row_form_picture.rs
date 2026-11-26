//! The list of the form view of `task-row` as a picture: sections, boxes, titles and the chips in columns, in both schemes.
//! Pictures are kept when `FS_COMP_SHOTS` names a directory.
//!
//! Pattern: Test double (a `NoWrite` host) over fixed rows. Pixel tests; they need a graphics adapter.

#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;

use common::{background, draw, ink, serial, set_dark};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::plan::form_view::View;
use fs_components_words::Locale;

fn row(key: &str, label: &str, done: &str, group: &str, facts: &[&str]) -> FormRow {
    let mut r = FormRow::of(key, &[("label", label), ("done", done), ("group", group)]);
    r.values.push((
        "facts".into(),
        facts.iter().map(|f| (*f).to_owned()).collect(),
    ));
    r
}

fn input() -> FormInput {
    FormInput {
        form: "checklist@1.1.0".to_owned(),
        rows: vec![
            row(
                "a",
                "Write the letter",
                "false",
                "today",
                &["High", "tomorrow"],
            ),
            row("b", "Buy bread", "true", "today", &["", "Fri"]),
            row(
                "c",
                "Call Anna about the trip",
                "false",
                "ready",
                &["Mid", ""],
            ),
            row("d", "Water the plants", "false", "ready", &[]),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

#[test]
fn the_list_draws_in_both_schemes_and_the_chips_stand_at_the_right() {
    let _g = serial();
    for dark in [false, true] {
        set_dark(dark);
        let ctx = DrawCtx::new(
            fs_components_core::form_view::PlaceInfo::default(),
            (StyleId::Admin, dark),
            Arc::new(|k| k.to_owned()),
            Locale::english(),
            Arc::new(NoWrite),
            Arc::new(input),
        );
        let mounted = View.mount(input(), ctx);
        let name = if dark {
            "task_row_list_dark.png"
        } else {
            "task_row_list_light.png"
        };
        let img = draw(mounted.build(640.0, 360.0), 640, 360, name);
        let bg = background(&img);
        assert!(ink(&img, (0, 0, 640, 360), bg) > 1500, "dark={dark}");
        assert!(
            ink(&img, (440, 40, 640, 140), bg) > 100,
            "chips at the right, dark={dark}"
        );
    }
    set_dark(false);
}
