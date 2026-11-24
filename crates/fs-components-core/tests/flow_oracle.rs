//! Taffy as the oracle for panels with `width == min` (dev only; taffy comes through `blinc_layout::style`).
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

mod common;

use blinc_layout::style::{
    AvailableSpace, Dimension, Display, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentageAuto, Size, Style, TaffyTree, length,
};
use common::{it, same, widths};
use fs_components_core::flow::FlowItem;
use proptest::prelude::*;

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
}
