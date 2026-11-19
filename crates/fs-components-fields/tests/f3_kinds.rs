//! The two kinds of bundle 3 as the registry and the contract see them: key, block, order, round trip through the standard type.
//!
//! Mutants: a kind that writes what `fs_types` refuses (the round test); a `Kind::block` that is not the contract's name
//! (`contract_renders.rs`); `utc_text` that keeps the offset.
#![allow(clippy::pedantic)]

use fs_components_fields::Kind;
use fs_components_fields::kinds::utc_text;
use fs_types::{BasicValue, DateTime, Duration};

#[test]
fn keys_and_blocks() {
    assert_eq!(Kind::DateTime.field().key(), "dateTime");
    assert_eq!(Kind::DateTime.block(), "field-date-time");
    assert_eq!(Kind::Duration.field().key(), "duration");
    assert_eq!(Kind::Duration.block(), "field-duration");
    assert_eq!(Kind::from_key("dateTime"), Some(Kind::DateTime));
    assert_eq!(Kind::from_block("field-duration"), Some(Kind::Duration));
}

#[test]
fn what_the_kinds_write_the_standard_reads_back_and_keeps() {
    let dt = Kind::DateTime.field();
    for text in [
        "2026-11-10T14:30:00+01:00",
        "2026-11-10T14:30:00Z",
        "2026-12-31T23:59:59-08:00",
    ] {
        let parts = dt.split(text);
        assert_eq!(dt.problem(&parts), None, "{text}");
        let joined = dt.join(&parts);
        assert_eq!(DateTime::parse(&joined).unwrap().canonical(), text);
    }
    let du = Kind::Duration.field();
    for text in ["P1W", "P1D", "PT45S", "P1DT2H30M", "PT1H0M5S"] {
        let parts = du.split(text);
        assert_eq!(du.problem(&parts), None, "{text}");
        assert_eq!(Duration::parse(&du.join(&parts)).unwrap().canonical(), text);
    }
}

#[test]
fn the_utc_form_moves_the_clock_and_drops_the_offset() {
    let parts = Kind::DateTime.field().split("2026-07-01T00:30:00+02:00");
    assert_eq!(
        utc_text(&parts).as_deref(),
        Some("2026-06-30T22:30:00.000Z")
    );
}
