//! The form `bars-in-time` as the timeline reads it: records to items, lanes, states, dependencies. No window.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;

use fs_components_timeline::layout::{BodyRow, body_rows};
use fs_components_timeline::*;
use host::{d, seed_records};

#[test]
fn the_timeline_names_the_form_in_the_three_part_spelling() {
    assert_eq!(bars::FORM, "bars-in-time@^1.0.0");
}

#[test]
fn lanes_stand_together_in_the_order_they_first_appear() {
    let mut records = seed_records();
    // a record of the first lane comes late in the list: it still joins its lane
    records.push(
        BarRecord::new("sofa", "Sofa", d(2026, 11, 2), d(2026, 11, 3)).lane("Umzug Hafenstraße"),
    );
    let items = BarsSource::new(records).items();
    let lanes: Vec<_> = items.iter().map(|i| i.lane.clone().unwrap()).collect();
    let mut seen: Vec<String> = Vec::new();
    for l in &lanes {
        if seen.last() != Some(l) {
            assert!(!seen.contains(l), "lane {l} is split");
            seen.push(l.clone());
        }
    }
    assert_eq!(seen, ["Umzug Hafenstraße", "Website-Relaunch", "Alltag"]);
    let rows = body_rows(&items);
    let heads = rows
        .iter()
        .filter(|r| matches!(r, BodyRow::Lane(_)))
        .count();
    assert_eq!(heads, 3);
    assert_eq!(rows.len(), items.len() + 3);
}

#[test]
fn no_lane_means_no_heading() {
    let items = vec![Item::bar("a", "A", d(2026, 1, 1), d(2026, 1, 2), 0)];
    assert_eq!(body_rows(&items), [BodyRow::Item(0)]);
}

#[test]
fn depends_become_arrows_from_the_predecessor_to_the_dependent() {
    let deps = BarsSource::new(seed_records()).dependencies();
    let pairs: Vec<_> = deps
        .iter()
        .map(|d| (d.from.as_str(), d.to.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [
            ("mietvertrag", "strom"),
            ("kartons", "einpacken"),
            ("startseite", "navigation")
        ]
    );
    // a depends on a record that is not in the list: no arrow, no panic
    let lone = BarRecord::new("x", "X", d(2026, 1, 1), d(2026, 1, 2)).depends_on("gone");
    assert!(BarsSource::new(vec![lone]).dependencies().is_empty());
}

/// Mutant guard: a bar WITHOUT its state must differ from one with it (colour slot and fill), whatever the code is.
#[test]
fn the_state_chooses_colour_slot_and_fill() {
    let item = |state: Option<&str>| {
        let mut r = BarRecord::new("t", "T", d(2026, 1, 1), d(2026, 1, 5));
        r.state = state.map(Into::into);
        BarsSource::new(vec![r]).items().remove(0)
    };
    let none = item(None);
    assert_eq!((none.tone, none.state.clone()), (Tone::Accent, None));
    for (code, tone) in [
        ("in-process", Tone::Accent),
        ("completed", Tone::Success),
        ("failed", Tone::Error),
        ("cancelled", Tone::Neutral),
        ("needs-action", Tone::Neutral),
        ("something-else", Tone::Neutral),
    ] {
        let i = item(Some(code));
        assert_eq!(i.tone, tone, "{code}");
        assert_eq!(i.state.as_deref(), Some(code));
    }
    assert_eq!(state_look("cancelled").fill, Fill::Outline);
    assert_eq!(state_look("completed").fill, Fill::Tint);
}

#[test]
fn a_completed_task_without_progress_counts_as_done() {
    let r = |s: &str| BarRecord::new("t", "T", d(2026, 1, 1), d(2026, 1, 5)).state(s);
    let pct = |r: BarRecord| match BarsSource::new(vec![r]).items()[0].kind {
        Kind::Bar { progress, .. } => progress,
        Kind::Milestone { .. } => unreachable!(),
    };
    assert_eq!(pct(r("completed")), 100);
    assert_eq!(pct(r("in-process")), 0);
    assert_eq!(pct(r("in-process").progress(35)), 35);
}

#[test]
fn an_end_before_the_start_is_one_day() {
    let it = BarsSource::new(vec![BarRecord::new("t", "T", d(2026, 1, 5), d(2026, 1, 1))]).items();
    assert_eq!(it[0].span(), (d(2026, 1, 5), d(2026, 1, 5)));
}
