//! Home and End of the agenda: Home chooses the FIRST entry and End the LAST, also when nothing is chosen yet.
//!
//! Pattern: Test double (a `NoWrite` host, fixed rows); the choice is seen through `refresh` (a choice the module no longer has is dropped).
//! They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;

use common::{layout, serial};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::DrawCtx;
use fs_components_core::form_write::NoWrite;
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::Day;
use fs_components_widgets::calendar_view::form_view::View;
use fs_components_words::Locale;

const W: f32 = 900.0;
const H: f32 = 640.0;

fn row(key: &str, day: &str) -> FormRow {
    FormRow::of(
        key,
        &[
            ("label", key),
            ("start", day),
            ("end", day),
            ("all-day", "true"),
        ],
    )
}

fn rows() -> Vec<FormRow> {
    vec![
        row("first", "2026-11-11"),
        row("middle", "2026-11-14"),
        row("last", "2026-11-20"),
    ]
}

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn mounted() -> Box<dyn fs_components_core::form_view::Mounted> {
    let ctx = DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(|| input(rows())),
    );
    let m = View.mount_on(&input(rows()), ctx, Day::new(2026, 11, 10).unwrap());
    let _tree = layout(m.build(W, H), W, H);
    m
}

/// Does the choice survive when the module drops the row `gone`?
fn survives_without(m: &dyn fs_components_core::form_view::Mounted, gone: &str) -> bool {
    m.refresh(input(
        rows().into_iter().filter(|r| r.key != gone).collect(),
    ));
    m.key(Key::Enter)
}

#[test]
fn home_without_a_choice_chooses_the_first_entry() {
    let _g = serial();
    let m = mounted();
    assert!(m.key(Key::Home));
    assert!(
        !survives_without(&*m, "first"),
        "the choice was the first entry"
    );
}

#[test]
fn end_without_a_choice_chooses_the_last_entry() {
    let _g = serial();
    let m = mounted();
    assert!(m.key(Key::End));
    assert!(
        !survives_without(&*m, "last"),
        "the choice was the last entry"
    );
}

#[test]
fn home_and_end_with_a_choice_jump_to_the_ends() {
    let _g = serial();
    let m = mounted();
    assert!(m.key(Key::Down));
    assert!(m.key(Key::Down));
    assert!(m.key(Key::Home));
    assert!(survives_without(&*m, "last"), "the first stays");
    assert!(m.key(Key::End));
    assert!(survives_without(&*m, "first"), "the last stays");
}
