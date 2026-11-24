//! Tests of `flow::lines`, `heights`, `side_by_side`: the hand table of spike S0 (8 cases), the rest of the rule, proptest
//! properties, and taffy as the oracle for panels with `width == min` (dev only; taffy comes through `blinc_layout::style`).
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use blinc_layout::style::{
    AvailableSpace, Dimension, Display, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentageAuto, Size, Style, TaffyTree, length,
};
use fs_components_core::fit::Fit;
use fs_components_core::flow::{FlowItem, FlowOpts, Justify, heights, lines, side_by_side};
use proptest::prelude::*;

const fn it(min: f32, width: f32, max: f32, grow: f32) -> FlowItem {
    FlowItem {
        min,
        width,
        max,
        grow,
    }
}

fn o(gap: f32) -> FlowOpts {
    FlowOpts {
        gap,
        justify: Justify::Center,
    }
}

fn widths(avail: f32, items: &[FlowItem], gap: f32) -> Vec<Vec<f32>> {
    lines(avail, items, &o(gap))
        .into_iter()
        .map(|l| l.widths)
        .collect()
}

fn same(a: &[Vec<f32>], b: &[Vec<f32>]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.len() == y.len() && x.iter().zip(y).all(|(p, q)| (p - q).abs() < 0.05))
}

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

// ---- taffy as the oracle (width == min) ----

fn taffy_lines(avail: f32, items: &[FlowItem], gap: f32) -> Vec<Vec<f32>> {
    let mut t: TaffyTree<()> = TaffyTree::new();
    t.disable_rounding();
    let kids: Vec<_> = items
        .iter()
        .map(|i| {
            t.new_leaf(Style {
                flex_basis: length(i.min),
                flex_grow: i.grow,
                flex_shrink: 1.0,
                min_size: Size {
                    width: length(i.min),
                    height: LengthPercentageAuto::auto(),
                },
                max_size: Size {
                    width: length(i.max),
                    height: LengthPercentageAuto::auto(),
                },
                size: Size {
                    width: Dimension::auto(),
                    height: length(10.0),
                },
                ..Default::default()
            })
            .unwrap()
        })
        .collect();
    let root = t
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: Some(JustifyContent::CENTER),
                gap: Size {
                    width: length(gap),
                    height: length(0.0),
                },
                size: Size {
                    width: length(avail),
                    height: Dimension::auto(),
                },
                ..Default::default()
            },
            &kids,
        )
        .unwrap();
    t.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(avail),
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
    let mut out: Vec<(f32, Vec<f32>)> = Vec::new();
    for k in kids {
        let l = t.layout(k).unwrap();
        match out.last_mut() {
            Some((y, line)) if (*y - l.location.y).abs() < 0.01 => line.push(l.size.width),
            _ => out.push((l.location.y, vec![l.size.width])),
        }
    }
    out.into_iter().map(|(_, l)| l).collect()
}

fn items_min_eq_width() -> impl Strategy<Value = Vec<FlowItem>> {
    prop::collection::vec((50u32..300, 0u32..400, 0u32..4), 1..9).prop_map(|v| {
        v.into_iter()
            .map(|(min, extra, grow)| {
                let min = min as f32;
                it(min, min, min + extra as f32, grow as f32)
            })
            .collect()
    })
}

proptest! {
    #[test]
    fn oracle_taffy_agrees_when_width_equals_min(items in items_min_eq_width(), avail in 300u32..1500, gap in 0u32..20) {
        let (avail, gap) = (avail as f32, gap as f32);
        let mine = widths(avail, &items, gap);
        let theirs = taffy_lines(avail, &items, gap);
        prop_assert!(same(&mine, &theirs), "mine {:?} taffy {:?}", mine, theirs);
    }

    #[test]
    fn lines_partition_in_order_and_fit(
        raw in prop::collection::vec((10u32..400, 0u32..300, 0u32..400, 0u32..4), 0..12),
        avail in 50u32..2000, gap in 0u32..30, j in 0u8..3,
    ) {
        let items: Vec<FlowItem> = raw.iter().map(|&(min, w, extra, g)| {
            let (min, w) = (min as f32, min as f32 + w as f32);
            it(min, w, w + extra as f32, g as f32)
        }).collect();
        let (avail, gap) = (avail as f32, gap as f32);
        let justify = [Justify::Start, Justify::Center, Justify::End][j as usize];
        let ls = lines(avail, &items, &FlowOpts { gap, justify });
        let mut next = 0;
        for l in &ls {
            prop_assert_eq!(l.first, next);
            prop_assert!(!l.widths.is_empty());
            next += l.widths.len();
            let used: f32 = l.widths.iter().sum::<f32>() + gap * (l.widths.len() as f32 - 1.0);
            prop_assert!(used <= avail + 0.01, "line wider than the frame: {} > {}", used, avail);
            prop_assert!(l.offset >= 0.0 && l.offset + used <= avail + 0.01);
            for (k, w) in l.widths.iter().enumerate() {
                let i = items[l.first + k];
                prop_assert!(*w <= i.max + 0.01);
                prop_assert!(*w >= i.min.min(avail) - 0.01);
            }
            // shrink before wrap: a line only ends early when the next panel's minimum really does not fit
            if let Some(n) = items.get(next) {
                let min_used: f32 = (l.first..next).map(|k| items[k].min).sum::<f32>() + gap * (l.widths.len() as f32 - 1.0);
                if l.widths.len() > 0 && items[l.first..next].iter().all(|i| i.min <= avail) {
                    prop_assert!(min_used + gap + n.min > avail - 0.01, "wrapped although the minimums fit");
                }
            }
        }
        prop_assert_eq!(next, items.len());
    }
}
