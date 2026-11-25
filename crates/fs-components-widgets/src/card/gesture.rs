//! The pointer gesture of a card: press, a few pixels, carry, release.
//!
//! Pattern: State machine, pure (Idle, Pressed, Dragging); moved here from the board (`fs-components-kanban/src/drag.rs`), where it was
//! tied to the board's own place type. The 4 px rule keeps a click a click: a card is opened by a click and carried only after the
//! pointer moved this far. The machine knows no Blinc and no container: what lies under the pointer is asked of [`Over`]
//! (Strategy), the keys, origins and places are the caller's types.

/// How far the pointer must move before a press becomes a carry, in pixels.
pub const THRESHOLD: f32 = 4.0;

/// Pattern: Strategy - what lies under the pointer. The container answers (a board with fixed sizes, a measured grid), the machine only asks.
pub trait Over<K, P> {
    /// The place at `x`, `y` (the second pair given to [`Gesture::pointer`]) when `skip` is carried; `None` = over no place.
    fn place(&self, x: f32, y: f32, skip: Option<&K>) -> Option<P>;
}

/// Where the gesture is. `K` = what is carried, `F` = where it came from, `P` = where it would land.
#[derive(Clone, Debug, PartialEq)]
pub enum Gesture<K, F, P> {
    /// Nothing.
    Idle,
    /// The button is down on a card, not moved far enough.
    Pressed {
        /// The card.
        card: K,
        /// Where it stands.
        from: F,
        /// The pointer when it went down (window coordinates).
        at: (f32, f32),
    },
    /// The card is carried.
    Dragging {
        /// The card.
        card: K,
        /// Where it came from.
        from: F,
        /// Where it would land now; `None` = over no place.
        over: Option<P>,
        /// The pointer, in the coordinates the caller gave as the second pair (for the picture of the card under it).
        pointer: (f32, f32),
    },
}

// Written out: `derive(Default)` would ask `K`, `F` and `P` to be `Default`, and a place need not be.
#[allow(clippy::derivable_impls)]
impl<K, F, P> Default for Gesture<K, F, P> {
    fn default() -> Self {
        Self::Idle
    }
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
pub enum Release<K, F, P> {
    /// Nothing was going on.
    Nothing,
    /// A click: the card is to be opened.
    Click(K),
    /// Put down at a place.
    Drop {
        /// The card.
        card: K,
        /// Where it came from.
        from: F,
        /// Where it lands.
        spot: P,
    },
    /// Put down over no place: the card stays where it was.
    Back,
}

impl<K: Clone, F: Clone, P> Gesture<K, F, P> {
    /// The button went down on `card`, standing at `from`.
    pub fn press(&mut self, card: impl Into<K>, from: impl Into<F>, at: (f32, f32)) {
        *self = Self::Pressed {
            card: card.into(),
            from: from.into(),
            at,
        };
    }

    /// The pointer moved to `window` (window coordinates, for the 4 px rule) which is `local` for the places `over` knows.
    pub fn pointer(
        &mut self,
        window: (f32, f32),
        local: (f32, f32),
        over: &impl Over<K, P>,
    ) -> Moved {
        let (card, from, started) = match self {
            Self::Idle => return Moved::Nothing,
            Self::Pressed { card, from, at } => {
                if (window.0 - at.0).hypot(window.1 - at.1) < THRESHOLD {
                    return Moved::Nothing;
                }
                (card.clone(), from.clone(), true)
            }
            Self::Dragging { card, from, .. } => (card.clone(), from.clone(), false),
        };
        let place = over.place(local.0, local.1, Some(&card));
        *self = Self::Dragging {
            card,
            from,
            over: place,
            pointer: local,
        };
        if started {
            Moved::Started
        } else {
            Moved::Carried
        }
    }

    /// The button went up.
    pub fn release(&mut self) -> Release<K, F, P> {
        match std::mem::take(self) {
            Self::Idle => Release::Nothing,
            Self::Pressed { card, .. } => Release::Click(card),
            Self::Dragging {
                card,
                from,
                over: Some(spot),
                ..
            } => Release::Drop { card, from, spot },
            Self::Dragging { over: None, .. } => Release::Back,
        }
    }

    /// Escape: give the card up. True if something was going on.
    pub fn cancel(&mut self) -> bool {
        !matches!(std::mem::take(self), Self::Idle)
    }

    /// The card carried now.
    pub fn carried(&self) -> Option<&K> {
        match self {
            Self::Dragging { card, .. } => Some(card),
            _ => None,
        }
    }
}
