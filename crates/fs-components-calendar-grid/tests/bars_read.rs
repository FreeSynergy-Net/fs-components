//! `bars-in-time` read as calendar entries: fixed rows, no module, no window.
//!
//! Tests: Table tests over `read_bars` (what is read, what is counted, what is never converted).

use fs_components_calendar_grid::calendar_core::{BarEntry, read_bars};
use fs_components_calendar_grid::{Day, Moment};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;

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

fn d(text: &str) -> Day {
    Day::parse(text).unwrap()
}

fn one(pairs: &[(&str, &str)]) -> Option<BarEntry> {
    read_bars(&input(vec![FormRow::of("r", pairs)]))
        .entries
        .pop()
}

#[test]
fn a_timed_row_keeps_the_wall_time_of_its_offset() {
    let e = one(&[
        ("label", "Standup"),
        ("start", "2026-11-25T09:30:00+01:00"),
        ("end", "2026-11-25T10:00:00+01:00"),
        ("lane", "team"),
        ("state", "busy"),
    ])
    .unwrap();
    assert_eq!(e.start, Moment::at(d("2026-11-25"), 9, 30));
    assert_eq!(e.end, Moment::at(d("2026-11-25"), 10, 0));
    assert!(!e.all_day);
    assert_eq!(e.label, "Standup");
    assert_eq!(e.lane.as_deref(), Some("team"));
    assert_eq!(e.state.as_deref(), Some("busy"));
}

#[test]
fn another_offset_is_not_converted() {
    let a = one(&[("start", "2026-07-01T23:15:00+09:00")]).unwrap();
    let b = one(&[("start", "2026-07-01T23:15:00Z")]).unwrap();
    assert_eq!(a.start, Moment::at(d("2026-07-01"), 23, 15));
    assert_eq!(a.start, b.start);
}

#[test]
fn a_full_date_is_all_day_and_its_last_day_is_shown() {
    let e = one(&[("start", "2026-02-27"), ("end", "2026-03-01")]).unwrap();
    assert!(e.all_day);
    assert_eq!(e.start, Moment::start_of(d("2026-02-27")));
    assert_eq!(e.end, Moment::start_of(d("2026-03-02")));
}

#[test]
fn the_flag_makes_a_timed_row_all_day_and_a_row_without_end_ends_at_its_start() {
    let e = one(&[("start", "2026-11-25T09:00:00Z"), ("all-day", "true")]).unwrap();
    assert!(e.all_day);
    assert_eq!(e.end, Moment::start_of(d("2026-11-26")));
    let m = one(&[("start", "2026-11-25T09:00:00Z")]).unwrap();
    assert_eq!(m.start, m.end);
}

#[test]
fn unreadable_rows_are_counted_and_the_others_stay_in_order() {
    let rows = vec![
        FormRow::of("a", &[("start", "2026-11-25")]),
        FormRow::of("no-start", &[("label", "x")]),
        FormRow::of("bad-date", &[("start", "2026-13-01")]),
        FormRow::of("bad-flag", &[("start", "2026-11-25"), ("all-day", "maybe")]),
        FormRow::of(
            "backwards",
            &[
                ("start", "2026-11-25T10:00:00Z"),
                ("end", "2026-11-25T09:00:00Z"),
            ],
        ),
        FormRow::of(
            "backwards-days",
            &[("start", "2026-11-26"), ("end", "2026-11-25")],
        ),
        FormRow::of(
            "mixed",
            &[("start", "2026-11-25T10:00:00Z"), ("end", "2026-11-26")],
        ),
        FormRow::of(
            "b",
            &[
                ("start", "2026-11-26T08:00:00Z"),
                ("end", "2026-11-26T09:00:00Z"),
            ],
        ),
    ];
    let read = read_bars(&input(rows));
    let keys: Vec<&str> = read.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, ["a", "b"]);
    assert_eq!(
        read.invalid,
        [
            "no-start",
            "bad-date",
            "bad-flag",
            "backwards",
            "backwards-days",
            "mixed"
        ]
    );
}

#[test]
fn a_row_without_label_shows_its_key() {
    assert_eq!(one(&[("start", "2026-11-25")]).unwrap().label, "r");
}
