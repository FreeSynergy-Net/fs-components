//! The sizes the board is drawn in: one column, the head of a column, the foot, the space between cards.
//!
//! Pattern: none - constants. Nothing here is a hit rectangle any more: the places a card can land are measured from the laid-out
//! tree (`fs-components-widgets` `card::MeasuredTargets`); these are only the widths the elements are made in.

/// Width of a column in pixels.
pub const COLUMN_W: f32 = 288.0;
/// Space around the content of a cell of the grid (the same as the padding of a head cell of the grid, so the heads and the cards stand on one line).
pub const PAD: f32 = 18.0;
/// Height of a column head (name, count, menu).
pub const HEAD_H: f32 = 48.0;
/// Height of the foot of a column (the quick entry).
pub const FOOT_H: f32 = 48.0;
/// Space between two cards.
pub const CARD_GAP: f32 = 10.0;
/// The least height of a cell, so an empty one can be hit.
pub const CELL_MIN_H: f32 = 72.0;
