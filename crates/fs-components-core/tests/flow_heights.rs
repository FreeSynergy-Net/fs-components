//! Tests of `flow::heights` and `side_by_side`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

mod common;

use common::{it, o};
use fs_components_core::fit::Fit;
use fs_components_core::flow::{heights, lines, side_by_side};

fn fit(h: f32, fill: bool) -> Fit {
    Fit {
        min: 1.0,
        width: 1.0,
        max: 1.0,
        grow: 1.0,
        min_h: h,
        height: h,
        fill,
        reflow_2d: false,
    }
}

#[test]
fn heights_share_the_rest_with_fill_lines_or_scroll() {
    let a = it(100.0, 100.0, 100.0, 1.0);
    let ls = lines(250.0, &[a, a, a], &o(0.0)); // [100 100] [100]
    let fits = [fit(50.0, false), fit(80.0, false), fit(40.0, true)];
    assert_eq!(heights(300.0, &ls, &fits), (vec![80.0, 220.0], false)); // 80 + 40 = 120; rest 180 to the fill line
    assert_eq!(heights(100.0, &ls, &fits), (vec![80.0, 40.0], true)); // does not fit: scrolls as a whole
    assert_eq!(heights(120.0, &ls, &fits), (vec![80.0, 40.0], false)); // exactly fits
}

#[test]
fn side_by_side_when_the_minimums_fit() {
    let a = it(200.0, 300.0, 400.0, 1.0);
    assert!(side_by_side(400.0, &[a, a], 0.0));
    assert!(!side_by_side(399.0, &[a, a], 0.0));
    assert!(!side_by_side(405.0, &[a, a], 6.0));
    assert!(side_by_side(406.0, &[a, a], 6.0));
}
