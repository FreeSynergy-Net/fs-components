//! Where a pointer is over a container of cards: which cell, and between which cards.
//!
//! Pattern: none - pure functions over rectangles. Blinc sends a drag only to the element that was pressed and has no drop-target event,
//! so somebody has to say what lies under the pointer; this is the rule, the rectangles come from the container (measured, see
//! `measured.rs`, or given in a test). The rule is the one of the old board (`fs-components-kanban/src/hit.rs`), kept: the index is
//! before the first card whose middle is below the pointer, a card exactly at its middle counts as above, the gap between two cells
//! belongs to none and both edges belong to the cell.

use super::refs::{CardRef, Target};

/// A rectangle in window coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// A rectangle.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// The point is inside, both edges count.
    pub fn contains(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.0 <= self.x + self.w && p.1 >= self.y && p.1 <= self.y + self.h
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// The y of the middle.
    pub fn middle(&self) -> f32 {
        self.y + self.h / 2.0
    }

    /// The same rectangle moved by `d` (how much the container scrolled since it was measured).
    #[must_use]
    pub fn shifted(&self, d: (f32, f32)) -> Self {
        Self::new(self.x + d.0, self.y + d.1, self.w, self.h)
    }
}

/// A cell that takes cards.
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    /// The lane, if the container has lanes.
    pub lane: Option<String>,
    /// The column.
    pub column: String,
    /// Where it is.
    pub rect: Rect,
}

/// A card shown in a cell.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    /// The card.
    pub card: CardRef,
    /// The lane of its cell.
    pub lane: Option<String>,
    /// The column of its cell.
    pub column: String,
    /// Where it is.
    pub rect: Rect,
}

fn in_zone<'a>(
    cards: &'a [Placed],
    z: &'a Zone,
    skip: Option<&'a CardRef>,
) -> impl Iterator<Item = &'a Placed> {
    cards
        .iter()
        .filter(move |c| c.lane == z.lane && c.column == z.column && Some(&c.card) != skip)
}

/// The place under the point `p` when `skip` is carried: the cell under it (none in a gap or outside) and the neighbours there
/// (without `skip`, in the order of `cards`).
pub fn target_at(
    zones: &[Zone],
    cards: &[Placed],
    p: (f32, f32),
    skip: Option<&CardRef>,
) -> Option<Target> {
    let z = zones.iter().find(|z| z.rect.contains(p))?;
    let shown: Vec<&Placed> = in_zone(cards, z, skip).collect();
    let index = shown.iter().take_while(|c| c.rect.middle() < p.1).count();
    Some(Target {
        lane: z.lane.clone(),
        column: z.column.clone(),
        after: index.checked_sub(1).map(|i| shown[i].card.clone()),
        before: shown.get(index).map(|c| c.card.clone()),
    })
}

/// The line that shows where a card would land at `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    /// Left edge (the cell's).
    pub x: f32,
    /// Where the line is.
    pub y: f32,
    /// Width (the cell's).
    pub w: f32,
}

/// The line of `to`: in the middle of the gap between `after` and `before`, under `after` when it is the last card, over `before`
/// when it is the first, the top of the cell when the cell is empty. `None` when the cell is not known.
pub fn line_at(zones: &[Zone], cards: &[Placed], to: &Target) -> Option<Line> {
    let z = zones
        .iter()
        .find(|z| z.lane == to.lane && z.column == to.column)?;
    let rect_of = |r: &CardRef| cards.iter().find(|c| &c.card == r).map(|c| c.rect);
    let y = match (
        to.after.as_ref().and_then(rect_of),
        to.before.as_ref().and_then(rect_of),
    ) {
        (Some(a), Some(b)) => f32::midpoint(a.bottom(), b.y),
        (Some(a), None) => a.bottom(),
        (None, Some(b)) => b.y,
        (None, None) => z.rect.y,
    };
    Some(Line {
        x: z.rect.x,
        y,
        w: z.rect.w,
    })
}
