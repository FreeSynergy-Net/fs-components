//! The room of the line of a card: how the width is shared between the pieces that may give way (the subtitle, the facts) so that
//! nothing is cut by the edge of the card.
//!
//! Pattern: Strategy (water-filling: the short pieces keep their width, the long ones share what is left) over pure functions. The
//! framework clips at the edge without a word ("ada, bob" became "ada, l", a single letter stood at the border); a piece that does not
//! get its width is cut here with an ellipsis (`fit_text` of the list widgets, not a second cutter), and the view gives it the whole
//! text as a tooltip.

use blinc_layout::text_measure::measure_text;
use fs_components_widgets::list_view::fit_text;

use crate::hit::COLUMN_W;

/// The width inside a card (column width less padding and border), where the line is drawn.
pub(crate) const LINE_ROOM: f32 = COLUMN_W - 28.0;

/// The size the text pieces of the line are drawn at (`.fs-board-card-meta`).
pub(crate) const META_SIZE: f32 = 13.0;

/// The width `text` takes in the line.
pub(crate) fn width_of(text: &str) -> f32 {
    measure_text(text, META_SIZE).width
}

/// `text` as it fits `room` pixels: whole, or cut with an ellipsis at its end.
pub(crate) fn fit(text: &str, room: f32) -> String {
    fit_text(text, room, META_SIZE)
}

/// The room each piece gets: `wanted` are the widths the pieces would like (in order), `room` is what they share. A piece that
/// wants no more than its fair share keeps what it wants; what that leaves is shared by the longer ones (shortest first).
pub(crate) fn share(wanted: &[f32], room: f32) -> Vec<f32> {
    let mut order: Vec<usize> = (0..wanted.len()).collect();
    order.sort_by(|a, b| wanted[*a].total_cmp(&wanted[*b]));
    let mut given = vec![0.0; wanted.len()];
    let mut left = room.max(0.0);
    for (done, i) in order.iter().enumerate() {
        let fair = left / (wanted.len() - done) as f32;
        given[*i] = wanted[*i].min(fair);
        left -= given[*i];
    }
    given
}

#[cfg(test)]
#[path = "card_fit_tests.rs"]
mod tests;
