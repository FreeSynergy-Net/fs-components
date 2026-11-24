//! Proptest property of `flow::lines`: partition in order, fit into the frame, shrink before wrap.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

mod common;

use common::it;
use fs_components_core::flow::{FlowItem, FlowOpts, Justify, lines};
use proptest::prelude::*;

proptest! {
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
                if !l.widths.is_empty() && items[l.first..next].iter().all(|i| i.min <= avail) {
                    prop_assert!(min_used + gap + n.min > avail - 0.01, "wrapped although the minimums fit");
                }
            }
        }
        prop_assert_eq!(next, items.len());
    }
}
