//! Tests of `flow::lines`: the hand table of spike S0 (8 cases) and the rest of the rule.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic, clippy::type_complexity)]

mod common;

use common::{it, o, same, widths};
use fs_components_core::flow::{FlowItem, FlowOpts, Justify, lines};

#[test]
fn the_hand_table_of_s0() {
    let a = it(200.0, 300.0, 400.0, 1.0);
    let third = 161.0 + 1.0 / 3.0;
    let table: Vec<(&str, f32, f32, Vec<FlowItem>, Vec<Vec<f32>>)> = vec![
        (
            "1 two fit their wish",
            600.0,
            0.0,
            vec![a, a],
            vec![vec![300.0, 300.0]],
        ),
        (
            "2 shrink below wish, one line",
            500.0,
            0.0,
            vec![a, a],
            vec![vec![250.0, 250.0]],
        ),
        (
            "3 wide: both to max",
            1000.0,
            0.0,
            vec![a, a],
            vec![vec![400.0, 400.0]],
        ),
        (
            "4 unequal wish, then grow",
            600.0,
            0.0,
            vec![it(100.0, 300.0, 600.0, 1.0), it(100.0, 200.0, 600.0, 1.0)],
            vec![vec![350.0, 250.0]],
        ),
        (
            "5 wrap by the minimum sum",
            450.0,
            0.0,
            vec![a, a, a],
            vec![vec![225.0, 225.0], vec![400.0]],
        ),
        (
            "6 gap 8, three, one line",
            500.0,
            8.0,
            vec![it(100.0, 150.0, 200.0, 1.0); 3],
            vec![vec![third; 3]],
        ),
        (
            "8 shrink first, wish 600 > 500, grow 0",
            500.0,
            0.0,
            vec![it(100.0, 300.0, 600.0, 0.0); 2],
            vec![vec![250.0, 250.0]],
        ),
    ];
    for (name, avail, gap, items, hand) in table {
        let got = widths(avail, &items, gap);
        assert!(same(&got, &hand), "{name}: got {got:?}, hand {hand:?}");
    }
}

#[test]
fn a_panel_wider_than_the_line_gets_the_line_not_a_scrollbar() {
    // S0 case 7 had [200] (overflow); the draft (part B, "Widerspruch") says: own line with exactly the available width.
    assert_eq!(
        widths(150.0, &[it(200.0, 300.0, 400.0, 1.0)], 0.0),
        vec![vec![150.0]]
    );
}

#[test]
fn a_single_panel_takes_the_whole_width_up_to_max() {
    let a = it(200.0, 300.0, 500.0, 1.0);
    assert_eq!(widths(450.0, &[a], 0.0), vec![vec![450.0]]);
    assert_eq!(widths(900.0, &[a], 0.0), vec![vec![500.0]]);
}

#[test]
fn shrink_before_wrap_is_the_difference_to_a_wish_wrap() {
    // Two panels wishing 300 in 500: a wish-wrap would give two lines; here one line of 250 + 250.
    let a = it(200.0, 300.0, 400.0, 1.0);
    assert_eq!(lines(500.0, &[a, a], &o(0.0)).len(), 1);
}

#[test]
fn justify_places_the_remainder() {
    let a = it(100.0, 100.0, 100.0, 1.0);
    let off = |j| {
        lines(
            500.0,
            &[a, a],
            &FlowOpts {
                gap: 0.0,
                justify: j,
            },
        )[0]
        .offset
    };
    assert_eq!(off(Justify::Start), 0.0);
    assert_eq!(off(Justify::Center), 150.0);
    assert_eq!(off(Justify::End), 300.0);
    assert_eq!(FlowOpts::default().justify, Justify::Center);
}

#[test]
fn max_freezes_and_the_rest_goes_on() {
    // A reaches max 150 early, B takes what is left.
    let items = [it(100.0, 100.0, 150.0, 1.0), it(100.0, 100.0, 1000.0, 1.0)];
    let w = &widths(600.0, &items, 0.0)[0];
    assert!(
        (w[0] - 150.0).abs() < 0.05 && (w[1] - 450.0).abs() < 0.05,
        "{w:?}"
    );
}

#[test]
fn first_indexes_and_edge_inputs() {
    let a = it(200.0, 200.0, 200.0, 1.0);
    let ls = lines(450.0, &[a, a, a, a, a], &o(0.0));
    assert_eq!(
        ls.iter()
            .map(|l| (l.first, l.widths.len()))
            .collect::<Vec<_>>(),
        vec![(0, 2), (2, 2), (4, 1)]
    );
    assert!(lines(500.0, &[], &o(0.0)).is_empty());
    // not a number / negative: no panic, no NaN
    let bad = it(f32::NAN, -5.0, f32::NAN, f32::INFINITY);
    for l in lines(f32::NAN, &[bad, bad], &o(f32::NAN)) {
        assert!(l.widths.iter().all(|w| w.is_finite()) && l.offset.is_finite());
    }
}

#[test]
fn an_unbounded_frame_counts_as_zero_and_a_huge_grow_does_not_overflow() {
    let a = it(0.0, 100.0, f32::INFINITY, 1.0);
    // documented: a non-finite avail is 0
    assert_eq!(widths(f32::INFINITY, &[a, a], 0.0), vec![vec![0.0, 0.0]]);
    // free * grow / sum used to overflow to inf with grow = f32::MAX and max = inf
    let big = it(0.0, 0.0, f32::INFINITY, f32::MAX);
    let w = &widths(1000.0, &[big, it(0.0, 0.0, f32::INFINITY, 1.0)], 0.0)[0];
    assert!(w.iter().all(|x| x.is_finite()), "{w:?}");
    assert!((w.iter().sum::<f32>() - 1000.0).abs() < 1.0, "{w:?}");
}
