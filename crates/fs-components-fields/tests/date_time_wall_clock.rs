//! `field-date-time` with RFC 8984 `LocalDateTime`: a value without zone is shown as written (no `Z`, no conversion); a zone or `Z`
//! in the value stays visible.
//!
//! Pattern: Strategy (the kind under test is one `BasicField`). Mutant: `join` appends `Z` for a blank offset -> the first test fails.
#![allow(clippy::pedantic)]

use fs_components_fields::Kind;
use fs_components_fields::Problem;
use fs_components_fields::kinds::utc_text;

#[test]
fn without_a_zone_nothing_is_added_or_converted() {
    let f = Kind::DateTime.field();
    let raw = "2026-11-25T14:30:00";
    let parts = f.split(raw);
    assert_eq!(parts[6], "", "no offset part");
    assert_eq!(f.problem(&parts), None);
    assert_eq!(f.join(&parts), raw);
    let shown = f.compact(&parts);
    assert_eq!(shown, raw, "input == output");
    assert!(!shown.contains('Z') && !shown.contains('+'));
    assert_eq!(utc_text(&parts), None, "no instant is invented");
}

#[test]
fn with_a_zone_the_zone_is_shown_and_z_stays_utc() {
    let f = Kind::DateTime.field();
    for raw in ["2026-11-25T14:30:00+01:00", "2026-11-25T14:30:00Z"] {
        assert_eq!(f.compact(&f.split(raw)), raw);
    }
    assert!(f.compact(&f.split("2026-11-25T14:30:00Z")).ends_with('Z'));
}

#[test]
fn a_wall_clock_with_a_bad_time_is_a_problem() {
    let f = Kind::DateTime.field();
    let mut v = f.split("2026-11-25T14:30:00");
    v[3] = "25".into();
    assert_eq!(f.problem(&v), Some(Problem::BadTime));
}
