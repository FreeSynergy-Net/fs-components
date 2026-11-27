//! Test support: carry a card with the pointer over the real layout (split from `mod.rs`).
#![allow(dead_code, clippy::pedantic)]

use super::Rig;
use fs_components_kanban::BoardBench;

/// Where in a cell a carried card is let go.
#[derive(Clone, Copy)]
pub enum Spot<'a> {
    /// At the top of the cell.
    Top,
    /// In the gap right under this card.
    After(&'a str),
    /// Under the last card.
    End,
}

/// Carry `card` with the pointer (press, move, let go) over the real layout of a board with the element id prefix `B`, to `spot`
/// of the cell `cell` (the id of the cell element, `B-col-doing`, `B-lane-ada-col-doing`). Rects come from the laid-out tree.
pub fn carry(bench: &BoardBench, card: &str, cell: &str, spot: Spot<'_>) {
    let mut rig = Rig::new(bench.clone(), 1500.0, 900.0);
    carry_in(&mut rig, &card_id(card), cell, spot);
}

/// The same on a rig that is already there, from the element `from`.
pub fn carry_in(rig: &mut Rig, from: &str, cell: &str, spot: Spot<'_>) {
    let (x, y, w, h) = rig.bounds(cell);
    let to_y = match spot {
        Spot::Top => y + 2.0,
        Spot::After(k) => {
            let (_, ky, _, kh) = rig.bounds(&card_id(k));
            ky + kh + 2.0
        }
        Spot::End => y + h - 2.0,
    };
    rig.drag_to(from, (x + w / 2.0, to_y));
}

/// Press on the card and carry it over `spot` of `cell`, without letting go (the card stays carried in `bench`).
pub fn hold(bench: &BoardBench, card: &str, cell: &str, spot: Spot<'_>) {
    let mut rig = Rig::new(bench.clone(), 1500.0, 900.0);
    let (x, y, w, h) = rig.bounds(cell);
    let to_y = match spot {
        Spot::Top => y + 2.0,
        Spot::After(k) => {
            let (_, ky, _, kh) = rig.bounds(&card_id(k));
            ky + kh + 2.0
        }
        Spot::End => y + h - 2.0,
    };
    let (fx, fy) = rig.centre(&card_id(card));
    rig.down(fx, fy);
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        rig.move_to(fx + (x + w / 2.0 - fx) * t, fy + (to_y - fy) * t);
    }
    rig.frame();
}

/// The element of a card: `a` in a plain board, `ada/a` in the lane `ada`.
pub fn card_id(card: &str) -> String {
    match card.split_once('/') {
        Some((lane, key)) => format!("B-lane-{lane}-card-{key}"),
        None => format!("B-card-{card}"),
    }
}
