//! The pointer gesture: press, a few pixels, carry, release.
//!
//! Pattern: State machine, pure (Idle, Pressed, Dragging). The 4 px rule keeps a click a click: a card is opened by a click and
//! carried only after the pointer moved this far. The machine knows no Blinc; the view feeds it the pointer.

use crate::hit::{Geometry, Spot};

/// How far the pointer must move before a press becomes a carry, in pixels.
pub const THRESHOLD: f32 = 4.0;

/// Where the gesture is.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Drag {
    /// Nothing.
    #[default]
    Idle,
    /// The button is down on a card, not moved far enough.
    Pressed {
        /// The card.
        card: String,
        /// The column it is in.
        from: String,
        /// The pointer when it went down (window coordinates).
        at: (f32, f32),
    },
    /// The card is carried.
    Dragging {
        /// The card.
        card: String,
        /// The column it came from.
        from: String,
        /// Where it would land now; `None` = over no column.
        over: Option<Spot>,
        /// The pointer in the coordinates of the columns row (for the picture of the card under it).
        pointer: (f32, f32),
    },
}

/// What a pointer movement did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// Nothing (idle, or still a press).
    Nothing,
    /// The carry began with this movement.
    Started,
    /// The carried card is over another place or the pointer moved.
    Carried,
}

/// What a release came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Release {
    /// Nothing was going on.
    Nothing,
    /// A click: the card is to be opened.
    Click(String),
    /// Put down at a place.
    Drop {
        /// The card.
        card: String,
        /// The column it came from.
        from: String,
        /// Where it lands.
        spot: Spot,
    },
    /// Put down over no column: the card stays where it was.
    Back,
}

impl Drag {
    /// The button went down on `card` in column `from`.
    pub fn press(&mut self, card: &str, from: &str, at: (f32, f32)) {
        *self = Drag::Pressed {
            card: card.to_owned(),
            from: from.to_owned(),
            at,
        };
    }

    /// The pointer moved to `window` (window coordinates, for the 4 px rule) which is `local` in the columns row.
    pub fn pointer(&mut self, window: (f32, f32), local: (f32, f32), geometry: &Geometry) -> Moved {
        let (card, from, started) = match self {
            Drag::Idle => return Moved::Nothing,
            Drag::Pressed { card, from, at } => {
                let far = (window.0 - at.0).hypot(window.1 - at.1);
                if far < THRESHOLD {
                    return Moved::Nothing;
                }
                (card.clone(), from.clone(), true)
            }
            Drag::Dragging { card, from, .. } => (card.clone(), from.clone(), false),
        };
        let over = geometry.spot(local.0, local.1, Some(&card));
        *self = Drag::Dragging {
            card,
            from,
            over,
            pointer: local,
        };
        if started {
            Moved::Started
        } else {
            Moved::Carried
        }
    }

    /// The button went up.
    pub fn release(&mut self) -> Release {
        match std::mem::take(self) {
            Drag::Idle => Release::Nothing,
            Drag::Pressed { card, .. } => Release::Click(card),
            Drag::Dragging {
                card,
                from,
                over: Some(spot),
                ..
            } => Release::Drop { card, from, spot },
            Drag::Dragging { over: None, .. } => Release::Back,
        }
    }

    /// Escape: give the card up.
    pub fn cancel(&mut self) -> bool {
        !matches!(std::mem::take(self), Drag::Idle)
    }

    /// The card carried now.
    pub fn carried(&self) -> Option<&str> {
        match self {
            Drag::Dragging { card, .. } => Some(card),
            _ => None,
        }
    }
}
