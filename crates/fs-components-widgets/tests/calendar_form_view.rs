//! The form view of `calendar-view` (the agenda): fixed `bars-in-time` rows (no module) through the registry entry, drawn offscreen.
//!
//! Pattern: Test double (a `NoWrite` host, a `say` that returns the key) over fixed rows of November 2026.
//! They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;

use blinc_layout::prelude::RenderTree;
use common::{Input, draw_tree, frame, layout, serial};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::Day;
use fs_components_widgets::calendar_view::form_view::{View, WIDGET};
use fs_components_widgets::calendar_view::{partial_notice, read_bars};
use fs_components_words::Locale;

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
    let t = |h: u8, m: u8| format!("2026-11-10T{h:02}:{m:02}:00+01:00");
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
        row("sync", "Sync", &t(10, 0), &t(11, 0), "team", "false"),
        row("call", "Call", &t(10, 30), &t(11, 30), "home", "false"),
        row("review", "Review", &t(13, 0), &t(14, 0), "team", "false"),
        row("lunch", "Lunch", &t(12, 0), &t(13, 0), "home", "false"),
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

fn texts(tree: &RenderTree) -> Vec<String> {
    tree.text_elements()
        .into_iter()
        .map(|(t, _)| t.content)
        .collect()
}

#[test]
fn the_view_is_the_widget_of_the_block_and_lists_the_entries_of_the_span_over_the_month_edge() {
    let _g = serial();
    assert_eq!(View.widget(), WIDGET);
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    // From the 10th on for 30 days: the trip is on its last three days, the 10th has four more, and the entry over the month
    // edge is on both of its days; what ended before the 10th and the all-day entry of the 3rd are not listed.
    for id in [
        "agenda-item-trip@2026-11-10",
        "agenda-item-trip@2026-11-12",
        "agenda-item-sync@2026-11-10",
        "agenda-item-holiday@2026-11-20",
        "agenda-item-in@2026-11-30",
        "agenda-item-in@2026-12-01",
    ] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    for id in [
        "agenda-item-offsite@2026-11-03",
        "agenda-item-out@2026-11-02",
    ] {
        assert!(tree.query_by_id(id).is_none(), "{id} is before the span");
    }
    assert!(tree.query_by_id("calendar-partial").is_none());
}

#[test]
fn nothing_in_the_span_says_so_with_the_word_of_the_host() {
    let _g = serial();
    let mounted = View.mount_on(&input(Vec::new()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    assert!(texts(&tree).iter().any(|t| t == "calendar.empty"));
}

#[test]
fn a_row_that_cannot_be_read_gives_a_drawn_notice_naming_it_and_the_rest_stays() {
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
    let mounted = View.mount_on(&input(all.clone()), ctx(false), today());
    let tree = layout(mounted.build(W, H), W, H);
    assert!(tree.query_by_id("calendar-partial").is_some(), "the notice");
    assert!(tree.query_by_id("agenda-item-trip@2026-11-10").is_some());
    let drawn = texts(&tree);
    assert!(
        drawn.iter().any(|t| t == "1 of 10 could not be shown"),
        "the drawn sentence: {drawn:?}"
    );
    assert!(drawn.iter().any(|t| t == "broken"), "the drawn name");
    // The same sentence the shared function gives.
    let read = read_bars(&input(all));
    assert_eq!(
        partial_notice(&ctx(false), &read.invalid, 10).lead(),
        "1 of 10 could not be shown"
    );
}

#[test]
fn keys_walk_the_items_and_page_keys_move_the_span() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let _tree = layout(mounted.build(W, H), W, H);
    assert!(!mounted.key(Key::Enter), "nothing is chosen yet");
    assert!(mounted.key(Key::Down), "the first item");
    assert!(mounted.key(Key::Enter), "a choice is taken");
    assert!(mounted.key(Key::End), "the last item");
    assert!(!mounted.key(Key::Escape), "an unknown key goes on");
    assert!(mounted.key(Key::PageDown), "the next 30 days (10 Dec on)");
    let tree = layout(mounted.build(W, H), W, H);
    assert!(tree.query_by_id("agenda-item-trip@2026-11-10").is_none());
    assert!(
        tree.query_by_id("agenda-item-in@2026-12-01").is_none(),
        "10 Dec on: nothing left"
    );
    assert!(texts(&tree).iter().any(|t| t == "calendar.empty"));
    assert!(!mounted.key(Key::Enter), "a new span has no choice");
}

#[test]
fn refresh_keeps_the_choice_the_span_and_drops_a_choice_that_is_gone() {
    let _g = serial();
    let mounted = View.mount_on(&input(rows()), ctx(false), today());
    let mut tree = layout(mounted.build(W, H), W, H);
    let mut pointer = Input::new();
    pointer.click(&mut tree, "agenda-item-sync@2026-11-10");
    frame(&mut tree, W, H);
    assert!(mounted.key(Key::Enter), "the click chose an item");
    mounted.refresh(input(rows()[2..].to_vec()));
    assert!(mounted.key(Key::Enter), "the item is still there");
    mounted.refresh(input(vec![rows()[0].clone()]));
    assert!(!mounted.key(Key::Enter), "the item is gone");
    assert!(mounted.key(Key::PageDown));
    mounted.refresh(input(rows()));
    let tree = layout(mounted.build(W, H), W, H);
    assert!(
        tree.query_by_id("agenda-item-trip@2026-11-10").is_none(),
        "the span stays where it was moved to"
    );
}

#[test]
fn the_agenda_is_drawn_light_and_dark() {
    let _g = serial();
    for (dark, name) in [
        (false, "calendar_agenda_light.png"),
        (true, "calendar_agenda_dark.png"),
    ] {
        common::set_dark(dark);
        let mounted = View.mount_on(&input(rows()), ctx(dark), today());
        let tree = layout(mounted.build(W, H), W, H);
        if common::has_fonts() {
            let img = draw_tree(&tree, W as u32, H as u32, name);
            assert!(
                common::ink(&img, (0, 0, W as u32, H as u32), common::background(&img)) > 500,
                "{name}"
            );
        }
    }
    common::set_dark(false);
}
