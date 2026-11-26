//! ISO 8601 week numbers: the function, the switch of the place and the month grid with the column.
//!
//! Pattern: Table test — days and their weeks, places and their switch, then the drawn grid with the switch on and off.
//! The drawing needs a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

mod support;

use std::sync::Arc;

use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::View;
use fs_components_calendar_grid::iso_week::{iso_week, row_week};
use fs_components_calendar_grid::place::week_numbers_of;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use support::texts_under;
use ui::{draw_tree, layout, serial};

fn day(y: i32, m: u8, d: u8) -> Day {
    Day::new(y, m, d).unwrap()
}

#[test]
fn iso_weeks_follow_the_standard_at_the_year_edges() {
    let table = [
        (day(2026, 1, 1), 1),   // Thursday: week 1 of 2026
        (day(2025, 12, 29), 1), // Monday of that week, still in 2025
        (day(2026, 12, 31), 53),
        (day(2021, 1, 3), 53), // Sunday, belongs to 2020
        (day(2021, 1, 4), 1),
        (day(2024, 12, 30), 1), // Monday, belongs to 2025
        (day(2024, 2, 29), 9),  // leap day
        (day(2026, 11, 26), 48),
    ];
    for (d, week) in table {
        assert_eq!(iso_week(d), Some(week), "{d:?}");
    }
}

#[test]
fn a_row_stands_for_the_week_of_its_thursday_whatever_its_first_day() {
    // Monday start and Sunday start around 2026-01-01 (Thursday): both rows hold that Thursday.
    let monday: Vec<Day> = (0..7).map(|i| day(2025, 12, 29).add_days(i)).collect();
    let sunday: Vec<Day> = (0..7).map(|i| day(2025, 12, 28).add_days(i)).collect();
    assert_eq!(row_week(&monday), Some(1));
    assert_eq!(row_week(&sunday), Some(1));
    assert_eq!(row_week(&monday[..3]), None, "no Thursday, no week");
}

#[test]
fn the_switch_is_the_contract_default_unless_the_place_says_otherwise() {
    let on = |v: &str| week_numbers_of(&PlaceInfo::default().with_option("view.week-numbers", v));
    assert!(!week_numbers_of(&PlaceInfo::default()));
    assert!(on("true"));
    assert!(!on("false"));
    for word in ["", "yes", "True", "1"] {
        assert!(!on(word), "{word:?}");
    }
}

fn input() -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![FormRow::of(
            "offsite",
            &[
                ("label", "Offsite"),
                ("start", "2026-11-09"),
                ("end", "2026-11-11"),
                ("lane", "team"),
                ("all-day", "true"),
            ],
        )],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(place: PlaceInfo, dark: bool) -> DrawCtx {
    DrawCtx::new(
        place,
        (StyleId::Admin, dark),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

fn place(on: bool) -> PlaceInfo {
    PlaceInfo::default().with_option("view.week-numbers", if on { "true" } else { "false" })
}

/// (week rows, cells) of the month the place draws.
fn counts(place: PlaceInfo) -> (usize, usize) {
    let _g = serial();
    let mounted = View.mount_on(&input(), ctx(place, false), day(2026, 11, 10));
    let tree = layout(mounted.build(900.0, 640.0), 900.0, 640.0);
    let n = |prefix: &str| {
        (0..42)
            .filter(|i| tree.query_by_id(&format!("{prefix}-{i}")).is_some())
            .count()
    };
    (n("cal-week"), n("cal-cell"))
}

#[test]
fn six_week_numbers_only_when_the_place_asks_and_the_cells_stay() {
    assert_eq!(counts(PlaceInfo::default()), (0, 42));
    assert_eq!(counts(place(false)), (0, 42));
    assert_eq!(counts(place(true)), (6, 42));
    // The same block in the week view has no column: the decorator is the month's.
    assert_eq!(counts(place(true).with_view("week")), (0, 0));
}

#[test]
fn the_column_says_the_iso_weeks_of_november_2026() {
    let _g = serial();
    let mounted = View.mount_on(&input(), ctx(place(true), false), day(2026, 11, 10));
    let tree = layout(mounted.build(900.0, 640.0), 900.0, 640.0);
    for (row, week) in [(0, "44"), (1, "45"), (4, "48"), (5, "49")] {
        assert_eq!(
            texts_under(&tree, &format!("cal-week-{row}")),
            [week],
            "row {row}"
        );
    }
}

#[test]
fn the_month_is_drawn_with_week_numbers_light_and_dark() {
    let _g = serial();
    let (w, h) = (900.0_f32, 640.0_f32);
    for dark in [false, true] {
        ui::set_dark(dark);
        let name = format!(
            "calendar_week_numbers_{}.png",
            if dark { "dark" } else { "light" }
        );
        let mounted = View.mount_on(&input(), ctx(place(true), dark), day(2026, 11, 10));
        let tree = layout(mounted.build(w, h), w, h);
        if ui::has_fonts() {
            let img = draw_tree(&tree, w as u32, h as u32, &name);
            // The ink of the column itself, not of the whole picture: a column that is not drawn must fail.
            let node = tree.query_by_id("cal-week-numbers").expect("the column");
            let b = tree.get_absolute_bounds(node).expect("bounds");
            let rect = (b.x as u32, b.y as u32, b.width as u32, b.height as u32);
            assert!(ui::ink(&img, rect, ui::background(&img)) > 100, "{name}");
        }
    }
    ui::set_dark(false);
}
