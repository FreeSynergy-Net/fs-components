//! `Limit`: a count from one to `Limit::MAX`, checked in its constructor; `LimitSetting` is plain data (rule 57: the block takes
//! the limit as a parameter and reads no settings).
//!
//! Mutants: `Limit::new(0)` is a limit -> `zero_is_no_limit`; the ceiling is missing -> `the_ceiling_is_the_settings_whole_number`;
//! `of` accepts 0 -> `a_constant_outside_the_range_panics`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use fs_components_fields::{Limit, LimitSetting};

#[test]
fn zero_is_no_limit() {
    assert_eq!(Limit::new(0), None);
    assert_eq!(Limit::new(1).map(Limit::get), Some(1));
    assert_eq!(Limit::new(Limit::MAX).map(Limit::get), Some(Limit::MAX));
}

#[test]
fn the_ceiling_is_the_settings_whole_number() {
    assert_eq!(Limit::MIN, 1);
    assert_eq!(Limit::MAX, i32::MAX as usize);
    assert_eq!(Limit::new(Limit::MAX + 1), None);
    assert_eq!(Limit::new(usize::MAX), None);
}

#[test]
#[should_panic(expected = "at least one")]
fn a_constant_of_zero_panics() {
    let _ = Limit::of(std::hint::black_box(0));
}

#[test]
#[should_panic(expected = "at most MAX")]
fn a_constant_above_the_ceiling_panics() {
    let _ = Limit::of(std::hint::black_box(Limit::MAX + 1));
}

#[test]
fn a_setting_is_data() {
    const S: LimitSetting = LimitSetting {
        key: "fs.probe.count",
        default: Limit::of(9),
        group: "probe",
        label_key: "setting-label-fs-probe-count",
        about_key: "setting-about-fs-probe-count",
    };
    assert_eq!((S.key, S.default.get()), ("fs.probe.count", 9));
}
