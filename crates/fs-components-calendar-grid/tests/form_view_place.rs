//! The form view reads `ctx.place`: month, week and day from the same block, drawn offscreen in light and dark.
//!
//! Pattern: Table test over the places (None, month, week, day, unknown) with fixed rows of November 2026.
//! They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::View;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use ui::{draw_tree, layout, serial};

const W: f32 = 900.0;
const H: f32 = 640.0;

fn row(key: &str, label: &str, start: &str, end: &str, all_day: &str) -> FormRow {
    FormRow::of(
        key,
        &[
            ("label", label),
            ("start", start),
            ("end", end),
            ("lane", "team"),
            ("all-day", all_day),
        ],
    )
}

fn rows() -> Vec<FormRow> {
    vec![
        row("offsite", "Offsite", "2026-11-09", "2026-11-11", "true"),
        row(
            "sync",
            "Sync",
            "2026-11-10T10:00:00+01:00",
            "2026-11-10T11:00:00+01:00",
            "false",
        ),
        row(
            "call",
            "Call",
            "2026-11-10T10:30:00+01:00",
            "2026-11-10T11:30:00+01:00",
            "false",
        ),
        row(
            "lunch",
            "Lunch",
            "2026-11-12T12:00:00+01:00",
            "2026-11-12T13:00:00+01:00",
            "false",
        ),
    ]
}

fn input() -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: rows(),
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(place: PlaceInfo, dark: bool) -> DrawCtx {
    DrawCtx {
        place,
        look: (StyleId::Admin, dark),
        // A host without sentences: the grid must still show words.
        say: Arc::new(|k| k.to_owned()),
        locale: Locale::english(),
        write: Arc::new(NoWrite),
        reread: Arc::new(input),
    }
}

/// (month cells, all-day cells) the place draws.
fn counts(place: PlaceInfo) -> (usize, usize) {
    let _g = serial();
    let mounted = View.mount_on(&input(), ctx(place, false), Day::new(2026, 11, 10).unwrap());
    let tree = layout(mounted.build(W, H), W, H);
    let n = |prefix: &str| {
        (0..42)
            .filter(|i| tree.query_by_id(&format!("{prefix}-{i}")).is_some())
            .count()
    };
    (n("cal-cell"), n("cal-allday"))
}

#[test]
fn no_place_and_unknown_views_draw_the_month() {
    assert_eq!(counts(PlaceInfo::default()), (42, 0));
    assert_eq!(counts(PlaceInfo::default().with_view("agenda")), (42, 0));
    assert_eq!(counts(PlaceInfo::default().with_view("nonsense")), (42, 0));
    assert_eq!(counts(PlaceInfo::default().with_view("month")), (42, 0));
}

#[test]
fn week_draws_seven_columns_and_day_draws_one() {
    assert_eq!(counts(PlaceInfo::default().with_view("week")), (0, 7));
    assert_eq!(counts(PlaceInfo::default().with_view("day")), (0, 1));
}

#[test]
fn week_and_day_hold_the_items_of_their_range_in_the_viewed_zone() {
    let _g = serial();
    let today = Day::new(2026, 11, 10).unwrap();
    let week = View.mount_on(
        &input(),
        ctx(PlaceInfo::default().with_view("week"), false),
        today,
    );
    let tree = layout(week.build(W, H), W, H);
    // The 9th to 11th (Mon to Wed) is one bar over three columns; the call of the 10th is in a column.
    assert!(
        tree.query_by_id("cal-item-offsite").is_some(),
        "the all-day bar"
    );
    assert!(tree.query_by_id("cal-item-sync").is_some());
    let day = View.mount_on(
        &input(),
        ctx(PlaceInfo::default().with_view("day"), false),
        today,
    );
    let tree = layout(day.build(W, H), W, H);
    assert!(tree.query_by_id("cal-item-sync").is_some(), "the 10th");
    assert!(tree.query_by_id("cal-item-call").is_some(), "the 10th");
    assert!(
        tree.query_by_id("cal-item-lunch").is_none(),
        "the 12th is not on the day"
    );
}

#[test]
fn every_view_is_drawn_light_and_dark() {
    let _g = serial();
    let today = Day::new(2026, 11, 10).unwrap();
    for view in ["month", "week", "day"] {
        for dark in [false, true] {
            ui::set_dark(dark);
            let name = format!(
                "calendar_{view}_{}.png",
                if dark { "dark" } else { "light" }
            );
            let mounted = View.mount_on(
                &input(),
                ctx(PlaceInfo::default().with_view(view), dark),
                today,
            );
            let tree = layout(mounted.build(W, H), W, H);
            if ui::has_fonts() {
                let img = draw_tree(&tree, W as u32, H as u32, &name);
                assert!(
                    ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500,
                    "{name}"
                );
            }
        }
    }
    ui::set_dark(false);
}
