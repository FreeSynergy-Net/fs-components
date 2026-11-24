//! Tests of `fit::merge`: order of the layers, clamps and their notes, em, fill.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use fs_components_core::fit::{FitNote, Len, SizeSpec, merge};

fn spec(min: f32, width: f32, max: f32) -> SizeSpec {
    SizeSpec {
        min: Some(Len::Lpx(min)),
        width: Some(Len::Lpx(width)),
        max: Some(Len::Lpx(max)),
        ..Default::default()
    }
}
const NONE: SizeSpec = SizeSpec {
    min: None,
    width: None,
    max: None,
    grow: None,
    min_h: None,
    height: None,
};

#[test]
fn contract_alone_and_defaults() {
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &NONE, &NONE, 0.0, 16.0);
    assert!(n.is_empty());
    assert_eq!(
        (f.min, f.width, f.max, f.grow, f.min_h, f.height, f.fill),
        (200.0, 300.0, 400.0, 1.0, 0.0, 0.0, false)
    );
    let (f, _) = merge(&NONE, &NONE, &NONE, 44.0, 16.0); // theme gives only a lower bound; width follows min
    assert_eq!((f.min, f.width), (44.0, 44.0));
    assert!(f.max.is_infinite());
}

#[test]
fn em_is_resolved_once_with_the_given_em() {
    let c = SizeSpec {
        min: Some(Len::Em(10.0)),
        width: Some(Len::Em(20.0)),
        max: Some(Len::Em(40.0)),
        ..Default::default()
    };
    let (f, _) = merge(&c, &NONE, &NONE, 0.0, 18.0);
    assert_eq!((f.min, f.width, f.max), (180.0, 360.0, 720.0));
}

#[test]
fn layout_may_narrow_and_raise_min() {
    let l = SizeSpec {
        min: Some(Len::Lpx(250.0)),
        max: Some(Len::Lpx(350.0)),
        width: Some(Len::Lpx(320.0)),
        ..Default::default()
    };
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &l, &NONE, 0.0, 16.0);
    assert!(n.is_empty());
    assert_eq!((f.min, f.width, f.max), (250.0, 320.0, 350.0));
}

#[test]
fn layout_beyond_the_contract_is_clamped_with_a_note() {
    let l = SizeSpec {
        min: Some(Len::Lpx(100.0)),
        max: Some(Len::Lpx(900.0)),
        width: Some(Len::Lpx(800.0)),
        ..Default::default()
    };
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &l, &NONE, 0.0, 16.0);
    assert_eq!((f.min, f.width, f.max), (200.0, 300.0, 400.0));
    for key in ["min-width", "max-width", "width"] {
        assert!(
            n.contains(&FitNote::LayoutWiderThanContract { key }),
            "{key}: {n:?}"
        );
    }
}

#[test]
fn the_lower_bound_wins_over_the_upper() {
    let l = SizeSpec {
        min: Some(Len::Lpx(500.0)),
        ..Default::default()
    };
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &l, &NONE, 0.0, 16.0);
    assert!(n.contains(&FitNote::MinAboveMax));
    assert_eq!((f.min, f.max, f.width), (500.0, 500.0, 500.0));
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &NONE, &NONE, 450.0, 16.0); // theme above the contract's max
    assert!(n.contains(&FitNote::MinAboveMax));
    assert_eq!((f.min, f.max), (450.0, 450.0));
}

#[test]
fn the_person_sets_only_the_wish_and_is_clamped() {
    let p = |w| SizeSpec {
        width: Some(Len::Lpx(w)),
        min: Some(Len::Lpx(1.0)),
        max: Some(Len::Lpx(9999.0)),
        ..Default::default()
    };
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &NONE, &p(350.0), 0.0, 16.0);
    assert!(n.is_empty());
    assert_eq!((f.min, f.width, f.max), (200.0, 350.0, 400.0)); // person's min/max ignored
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &NONE, &p(900.0), 0.0, 16.0);
    assert_eq!(f.width, 400.0);
    assert!(n.contains(&FitNote::PersonClamped));
    let (f, n) = merge(&spec(200.0, 300.0, 400.0), &NONE, &p(10.0), 0.0, 16.0);
    assert_eq!(f.width, 200.0);
    assert!(n.contains(&FitNote::PersonClamped));
}

#[test]
fn grow_height_and_fill() {
    let c = SizeSpec {
        grow: Some(2.0),
        min_h: Some(Len::Em(10.0)),
        ..Default::default()
    };
    let l = SizeSpec {
        height: Some(Len::Fill),
        min_h: Some(Len::Lpx(100.0)),
        ..Default::default()
    };
    let (f, _) = merge(&c, &l, &NONE, 0.0, 16.0);
    assert!(f.fill && !f.reflow_2d);
    assert_eq!((f.grow, f.min_h, f.height), (2.0, 160.0, 160.0));
    let l = SizeSpec {
        grow: Some(0.0),
        height: Some(Len::Lpx(400.0)),
        ..Default::default()
    };
    let (f, _) = merge(&c, &l, &NONE, 0.0, 16.0);
    assert_eq!((f.grow, f.height, f.fill), (0.0, 400.0, false));
    let l = SizeSpec {
        height: Some(Len::Lpx(50.0)),
        ..Default::default()
    }; // never below min_h
    assert_eq!(merge(&c, &l, &NONE, 0.0, 16.0).0.height, 160.0);
}

#[test]
fn bad_numbers_say_nothing() {
    let c = SizeSpec {
        min: Some(Len::Lpx(f32::NAN)),
        width: Some(Len::Lpx(-4.0)),
        max: Some(Len::Fit),
        grow: Some(f32::NAN),
        ..Default::default()
    };
    let (f, n) = merge(&c, &NONE, &NONE, f32::NAN, 16.0);
    assert!(n.is_empty());
    assert_eq!((f.min, f.width, f.grow), (0.0, 0.0, 1.0));
    assert!(f.max.is_infinite());
}
