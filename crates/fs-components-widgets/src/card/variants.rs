//! The variants of the card (contract block `card`, `variants`): the look, never the behaviour.
//!
//! Pattern: Builder (the variant is a flag of the card, drawn by the class in `view.rs`; the gestures, the keys and the state are the
//! same for every variant).

use super::widget::Card;

impl Card {
    /// The variant `panel` (roomier, for a framed area that holds one thing).
    #[must_use]
    pub fn panel(mut self) -> Self {
        self.panel = true;
        self
    }

    /// The variant `shared-lines` (square, only the right and the bottom line: the next card in the row and the row below share
    /// them, the frame that holds the cards closes the top and the left line; the Craft look of a grid of tiles).
    #[must_use]
    pub fn shared_lines(mut self) -> Self {
        self.shared = true;
        self
    }
}
