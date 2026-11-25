//! The form view of `timeline`: fixed `bars-in-time` rows (no module) through the registry entry, drawn offscreen.
//!
//! Pattern: Test double (a `NoWrite` host, a `say` that returns the key) over fixed rows of November 2026.
//! They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use blinc_layout::prelude::RenderTree;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_timeline::form_view::{View, WIDGET};
use fs_components_widgets::calendar_view::{Day, partial_notice, read_bars};
use fs_components_words::Locale;
use ui::{Input, draw_tree, frame, layout, serial};

const W: f32 = 1000.0;
const H: f32 = 560.0;

fn row(key: &str, label: &str, start: &str, end: &str, lane: &str) -> FormRow {
    FormRow::of(
        key,
        &[
            ("label", label),
            ("start", start),
            ("end", end),
            ("lane", lane),
        ],
    )
}

fn rows() -> Vec<FormRow> {
    let mut over = row("over", "Over the edge", "2026-11-28", "2026-12-03", "team");
    over.values.push(("depends".into(), vec!["trip".into()]));
    let mut trip = row("trip", "Trip", "2026-11-09", "2026-11-12", "team");
    trip.values.push(("progress".into(), vec!["40".into()]));
    vec![
        trip,
        row(
            "sync",
            "Sync",
            "2026-11-10T10:00:00+01:00",
            "2026-11-10T11:00:00+01:00",
            "home",
        ),
        row(
            "go",
            "Go live",
            "2026-11-20T09:00:00+01:00",
            "2026-11-20T09:00:00+01:00",
            "home",
        ),
        over,
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

fn ctx(dark: bool) -> DrawCtx {
    DrawCtx {
        place: fs_components_core::form_view::PlaceInfo::default(),
        look: (StyleId::Admin, dark),
        say: Arc::new(|k| k.to_owned()),
        locale: Locale::english(),
        write: Arc::new(NoWrite),
        reread: Arc::new(|| input(rows())),
    }
}

fn today() -> Day {
    Day::new(2026, 11, 10).unwrap()
}

fn width_of(tree: &RenderTree, key: &str) -> f32 {
    tree.get_absolute_bounds(
        tree.query_by_id(&format!("tl-item-{key}"))
            .unwrap_or_else(|| panic!("no item {key}")),
    )
    .unwrap()
    .width
}

fn texts(tree: &RenderTree) -> Vec<String> {
    tree.text_elements()
        .into_iter()
        .map(|(t, _)| t.content)
        .collect()
}

#[test]
fn lanes_stand_together_and_a_multi_day_bar_is_longer_than_one_day() {
    let _g = serial();
    assert_eq!(View.widget(), WIDGET);
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    for id in ["tl-lane-team", "tl-lane-home", "tl-item-trip", "tl-item-go"] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    assert!(tree.query_by_id("timeline-partial").is_none());
    // Nov 9 to 12 inclusive is four days, a timed entry of one day is one.
    let (trip, sync) = (width_of(&tree, "trip"), width_of(&tree, "sync"));
    assert!(
        (trip / sync - 4.0).abs() < 0.1,
        "four days against one: {trip} / {sync}"
    );
    // The entry over the month edge (28 Nov to 3 Dec, six days) is one bar, not two.
    let over = width_of(&tree, "over");
    assert!((over / sync - 6.0).abs() < 0.1, "six days: {over}");
}

#[test]
fn a_record_of_no_duration_is_a_milestone_not_a_bar() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    // A bar has a drawn done part (`tl-done-<key>`), a milestone has none.
    assert!(tree.query_by_id("tl-done-trip").is_some());
    assert!(tree.query_by_id("tl-done-go").is_none());
    assert!(texts(&tree).iter().any(|t| t == "Go live"));
}

#[test]
fn a_row_that_cannot_be_read_gives_a_drawn_notice_naming_it_and_the_rest_stays() {
    let _g = serial();
    let mut all = rows();
    all.push(row("broken", "Broken", "not a date", "2026-11-11", "team"));
    let mounted = View.mount_on(&input(all.clone()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    assert!(tree.query_by_id("timeline-partial").is_some(), "the notice");
    assert!(tree.query_by_id("tl-item-trip").is_some());
    assert!(tree.query_by_id("tl-item-broken").is_none());
    let drawn = texts(&tree);
    assert!(
        drawn.iter().any(|t| t == "1 of 5 could not be shown"),
        "the drawn sentence: {drawn:?}"
    );
    assert!(drawn.iter().any(|t| t == "broken"), "the drawn name");
    let read = read_bars(&input(all));
    assert_eq!(
        partial_notice(&ctx(false), &read.invalid, 5).lead(),
        "1 of 5 could not be shown"
    );
}

#[test]
fn keys_walk_the_rows_and_plus_minus_zoom() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let _tree = layout(mounted.build(W, H), W, H);
    assert!(mounted.key(Key::Down), "the first row");
    assert!(mounted.key(Key::Down), "the next row");
    assert!(mounted.key(Key::End), "the last row");
    assert!(!mounted.key(Key::End), "already there");
    assert!(mounted.key(Key::Home), "the first row again");
    assert!(mounted.key(Key::Other(187)), "finer");
    assert!(!mounted.key(Key::Escape), "an unknown key goes on");
}

#[test]
fn refresh_keeps_the_choice_and_drops_a_choice_that_is_gone() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let mut tree = layout(mounted.build(W, H), W, H);
    let mut pointer = Input::new();
    pointer.click(&mut tree, "tl-item-trip");
    frame(&mut tree, W, H);
    assert!(!mounted.key(Key::Home), "the click chose the first row");
    let mut more = rows();
    more.push(row("extra", "Extra", "2026-11-15", "2026-11-16", "home"));
    mounted.refresh(input(more.clone()));
    let tree = layout(mounted.build(W, H), W, H);
    assert!(tree.query_by_id("tl-item-extra").is_some(), "the new row");
    assert!(
        !mounted.key(Key::Home),
        "the choice stayed on the first row"
    );
    more.retain(|r| r.key != "trip");
    mounted.refresh(input(more));
    assert!(
        mounted.key(Key::Home),
        "the choice is gone, Home chooses a row"
    );
}

#[test]
fn the_timeline_is_drawn_light_and_dark() {
    let _g = serial();
    for (dark, name) in [(false, "timeline_light.png"), (true, "timeline_dark.png")] {
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
