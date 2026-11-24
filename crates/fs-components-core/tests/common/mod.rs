//! Shared helpers of the flow tests.
#![allow(dead_code, clippy::pedantic)]

use fs_components_core::flow::{FlowItem, FlowOpts, Justify, lines};

pub const fn it(min: f32, width: f32, max: f32, grow: f32) -> FlowItem {
    FlowItem {
        min,
        width,
        max,
        grow,
    }
}

pub fn o(gap: f32) -> FlowOpts {
    FlowOpts {
        gap,
        justify: Justify::Center,
    }
}

pub fn widths(avail: f32, items: &[FlowItem], gap: f32) -> Vec<Vec<f32>> {
    lines(avail, items, &o(gap))
        .into_iter()
        .map(|l| l.widths)
        .collect()
}

pub fn same(a: &[Vec<f32>], b: &[Vec<f32>]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.len() == y.len() && x.iter().zip(y).all(|(p, q)| (p - q).abs() < 0.05))
}
