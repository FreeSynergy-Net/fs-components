//! The timeline takes the today line from `DrawCtx.clock`, not from the system clock: a `FixedClock` puts it on its day.
//!
//! Pattern: Test double (`FixedClock` injected through the context) over one row of November 2026.
//! Needs a graphics adapter like the other layout tests of this crate.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_core::clock::FixedClock;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_timeline::form_view::View;
use fs_components_words::Locale;
use ui::{layout, serial};

const W: f32 = 900.0;
const H: f32 = 560.0;

fn input() -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![FormRow::of(
            "r1",
            &[
                ("label", "Ship"),
                ("start", "2026-11-05"),
                ("end", "2026-11-12"),
                ("lane", "team"),
            ],
        )],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(days: i64) -> DrawCtx {
    DrawCtx::new(
        PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k: &str| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
    .with_clock(Arc::new(FixedClock::at_day(days)))
}

/// Days since 1970-01-01 of 2026-11-10 and of 2040-01-01 (far outside the axis of the row).
const NEAR: i64 = 20_767;
const FAR: i64 = 25_567;

fn has(days: i64, id: &str) -> bool {
    let _g = serial();
    let mounted = View.mount(input(), ctx(days));
    layout(mounted.build(W, H), W, H).query_by_id(id).is_some()
}

#[test]
fn the_today_line_stands_on_the_day_of_the_clock() {
    assert!(has(NEAR, "tl-today"), "the line, inside the axis");
    assert!(!has(NEAR, "tl-today-edge"));
}

#[test]
fn a_clock_far_away_gives_the_edge_mark_not_a_line() {
    assert!(has(FAR, "tl-today-edge"), "the edge mark with the arrow");
    assert!(!has(FAR, "tl-today"));
}
