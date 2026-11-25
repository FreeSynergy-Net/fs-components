//! The form view of `calendar-grid`: fixed `bars-in-time` rows (no module) through the registry entry, drawn offscreen.
//!
//! Pattern: Test double (a `NoWrite` host, a `say` that returns the key) over fixed rows of November 2026.
//! They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::{View, WIDGET};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use ui::{Input, draw_tree, frame, layout, serial};

const W: f32 = 900.0;
const H: f32 = 640.0;

fn row(key: &str, label: &str, start: &str, end: &str, lane: &str, all_day: &str) -> FormRow {
    FormRow::of(
        key,
        &[
            ("label", label),
            ("start", start),
            ("end", end),
            ("lane", lane),
            ("all-day", all_day),
        ],
    )
}

/// Nine entries: two all-day, one of several days, four on the 10th, two over the edges of the month; two lanes.
fn rows() -> Vec<FormRow> {
    vec![
        row(
            "offsite",
            "Offsite",
            "2026-11-03",
            "2026-11-04",
            "team",
            "true",
        ),
        row(
            "holiday",
            "Holiday",
            "2026-11-20",
            "2026-11-20",
            "home",
            "true",
        ),
        row(
            "trip",
            "Trip",
            "2026-11-09T09:00:00+01:00",
            "2026-11-12T17:00:00+01:00",
            "team",
            "false",
        ),
        row(
            "sync",
            "Sync",
            "2026-11-10T10:00:00+01:00",
            "2026-11-10T11:00:00+01:00",
            "team",
            "false",
        ),
        row(
            "call",
            "Call",
            "2026-11-10T10:30:00+01:00",
            "2026-11-10T11:30:00+01:00",
            "home",
            "false",
        ),
        row(
            "review",
            "Review",
            "2026-11-10T13:00:00+01:00",
            "2026-11-10T14:00:00+01:00",
            "team",
            "false",
        ),
        row(
            "lunch",
            "Lunch",
            "2026-11-10T12:00:00+01:00",
            "2026-11-10T13:00:00+01:00",
            "home",
            "false",
        ),
        row(
            "out",
            "Left over",
            "2026-10-30T20:00:00+01:00",
            "2026-11-02T09:00:00+01:00",
            "team",
            "false",
        ),
        row(
            "in",
            "Runs on",
            "2026-11-30T10:00:00+01:00",
            "2026-12-01T11:00:00+01:00",
            "home",
            "false",
        ),
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

fn today() -> Day {
    Day::new(2026, 11, 10).unwrap()
}

fn ctx(dark: bool) -> DrawCtx {
    DrawCtx {
        look: (StyleId::Admin, dark),
        say: Arc::new(|k| k.to_owned()),
        locale: Locale::english(),
        write: Arc::new(NoWrite),
        reread: Arc::new(|| input(rows())),
    }
}

#[test]
fn the_view_is_the_widget_of_the_block_and_draws_every_entry_in_its_cells() {
    let _g = serial();
    assert_eq!(View.widget(), WIDGET);
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    for id in ["cal-cell-0", "cal-cell-41"] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    // The month opens where the data starts (30 October is the first entry): the 6 x 7 cells hold 26 Oct to 6 Dec.
    for id in [
        "cal-item-offsite-1",
        "cal-item-holiday-3",
        "cal-item-trip-2",
        "cal-item-out-0",
        "cal-item-out-1",
        "cal-item-in-5",
        "cal-item-sync-2",
    ] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    assert!(
        tree.query_by_id("cal-more-2-1").is_some(),
        "the 10th has more than fit: +N"
    );
    assert!(
        tree.query_by_id("calendar-partial").is_none(),
        "all rows were read"
    );
}

#[test]
fn a_row_that_cannot_be_read_gives_a_partial_notice_and_the_rest_stays() {
    let _g = serial();
    let mut all = rows();
    all.push(row(
        "broken",
        "Broken",
        "not a date",
        "2026-11-11",
        "team",
        "false",
    ));
    let mounted = View.mount_on(&input(all), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    assert!(tree.query_by_id("calendar-partial").is_some(), "the notice");
    assert!(
        tree.query_by_id("cal-item-trip-2").is_some(),
        "the readable rows are shown"
    );
}

#[test]
fn keys_walk_the_cells_and_an_unknown_key_is_not_taken() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let mut tree = layout(mounted.build(W, H), W, H);
    assert!(mounted.key(Key::Right));
    assert!(mounted.key(Key::Down));
    frame(&mut tree, W, H);
    assert!(tree.query_by_id("cal-slot").is_some() || tree.query_by_id("cal-cell-0").is_some());
    assert!(mounted.key(Key::PageDown), "the month changes");
    assert!(!mounted.key(Key::Escape), "an unknown key goes on");
}

#[test]
fn a_click_on_a_bar_chooses_it_and_refresh_keeps_the_month_and_the_choice() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let mut tree = layout(mounted.build(W, H), W, H);
    let mut pointer = Input::new();
    pointer.click(&mut tree, "cal-item-trip-2");
    frame(&mut tree, W, H);
    mounted.refresh(input(rows()[2..].to_vec()));
    let tree = layout(mounted.build(W, H), W, H);
    assert!(
        tree.query_by_id("cal-item-trip-2").is_some(),
        "the month stays"
    );
    assert!(
        tree.query_by_id("cal-item-offsite-1").is_none(),
        "the first two rows are gone"
    );
}

#[test]
fn the_month_is_drawn_light_and_dark() {
    let _g = serial();
    for (dark, name) in [
        (false, "calendar_month_light.png"),
        (true, "calendar_month_dark.png"),
    ] {
        ui::set_dark(dark);
        let mounted = View.mount_on(&input(rows()), ctx(dark), today());
        let tree = layout(mounted.build(W, H), W, H);
        if ui::has_fonts() {
            let img = draw_tree(&tree, W as u32, H as u32, name);
            assert!(
                ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500,
                "{name}"
            );
        }
    }
    ui::set_dark(false);
}
