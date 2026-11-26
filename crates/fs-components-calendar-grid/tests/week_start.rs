//! Pattern: Strategy (test) — the first weekday comes from the language (CLDR over `DrawCtx.dates`), the setting of the person
//! (`-u-fw`) or the place; the grid starts its week there and draws its heads in that order.
#![allow(clippy::pedantic)]

#[path = "support/dates.rs"]
mod dates;
#[path = "support/mod.rs"]
mod support;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use dates::{host_ctx, input};
use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::View;
use fs_components_calendar_grid::place::week_start_of;
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use ui::{layout, serial};

fn first_head(ctx: DrawCtx) -> String {
    let _g = serial();
    let mounted = View.mount_on(&input(vec![]), ctx, Day::new(2026, 11, 10).unwrap());
    let tree = layout(mounted.build(900.0, 640.0), 900.0, 640.0);
    support::texts_under(&tree, "cal-month")
        .into_iter()
        .next()
        .expect("a head of the month")
}

#[test]
fn the_week_starts_where_the_language_and_the_person_say() {
    let none = PlaceInfo::default();
    let start = |tag: &str, week: &str, place: &PlaceInfo| {
        week_start_of(
            place,
            &*host_ctx(tag, week, place.clone(), false, vec![]).dates,
        )
    };
    assert_eq!(start("de", "auto", &none), 0, "German: Monday");
    assert_eq!(start("en-US", "auto", &none), 6, "American English: Sunday");
    assert_eq!(start("de", "sun", &none), 6, "the setting wins");
    let sat = PlaceInfo::default().with_option("view.week-start", "sat");
    assert_eq!(start("de", "auto", &sat), 5, "the place wins");
    let odd = PlaceInfo::default().with_option("view.week-start", "someday");
    assert_eq!(
        start("de", "auto", &odd),
        0,
        "a word that is none: the language"
    );
}

#[test]
fn the_month_opens_with_the_first_weekday_of_the_reader() {
    let place = PlaceInfo::default;
    let de = first_head(host_ctx("de", "auto", place(), false, vec![]));
    let us = first_head(host_ctx("en-US", "auto", place(), false, vec![]));
    assert_eq!(de, "Mo");
    assert!(us.starts_with("Sun"), "{us}");
}
