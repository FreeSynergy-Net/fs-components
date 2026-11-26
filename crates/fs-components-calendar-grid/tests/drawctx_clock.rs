//! The grid takes "today" from `DrawCtx.clock`, not from the system clock: a `FixedClock` decides the month it opens on.
//!
//! Pattern: Test double (`FixedClock` injected through the context) over fixed rows of November 2026.
//! Needs a graphics adapter like the other layout tests of this crate.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::calendar_core::today;
use fs_components_calendar_grid::form_view::View;
use fs_components_core::clock::FixedClock;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use ui::{layout, serial};

const W: f32 = 900.0;
const H: f32 = 640.0;

fn input() -> FormInput {
    let row = FormRow::of(
        "party-1",
        &[
            ("label", "Party"),
            ("start", "2026-11-10T19:00:00+00:00"),
            ("end", "2026-11-10T22:00:00+00:00"),
            ("lane", "home"),
        ],
    );
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![row],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(day: Day) -> DrawCtx {
    DrawCtx::new(
        PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k: &str| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
    .with_clock(Arc::new(FixedClock::at_day(day.number())))
}

fn texts(day: Day) -> Vec<String> {
    let _g = serial();
    let mounted = View.mount(input(), ctx(day));
    layout(mounted.build(W, H), W, H)
        .text_elements()
        .into_iter()
        .map(|(t, _)| t.content)
        .collect()
}

#[test]
fn the_context_names_the_day_of_its_clock() {
    let day = Day::new(2026, 11, 10).unwrap();
    assert_eq!(today(&ctx(day)), day);
}

#[test]
fn the_month_opens_on_the_fixed_day_not_on_the_system_day() {
    let november = texts(Day::new(2026, 11, 10).unwrap());
    assert!(november.iter().any(|t| t == "2026-11"), "{november:?}");
    let march = texts(Day::new(2031, 3, 5).unwrap());
    assert!(march.iter().any(|t| t == "2031-03"), "{march:?}");
    assert!(!march.iter().any(|t| t == "2026-11"));
}
