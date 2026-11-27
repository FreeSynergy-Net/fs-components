//! The places of a container, read from the laid-out tree while a card is carried.
//!
//! Pattern: Adapter (the [`DropTargets`] strategy over measured rectangles) + Strategy ([`Bounds`]: where an element is; Blinc answers in
//! the window, a test answers with fixed rectangles). Measured in P0: `selector::query(id).bounds()` gives window coordinates during a
//! drag (error 0 px, about 30 us for 200 cards), BUT the cache is filled only when a layout runs: a scroll alone makes the rectangles
//! stale by exactly the way scrolled. So the adapter adds the scroll since the last layout ([`Scrolled`]; the container calls
//! [`MeasuredTargets::layout_done`] when a layout has run). Zoom is the container's layout size, never a transform (bounds and the
//! pointer ignore a transform).

use std::sync::{Arc, Mutex, PoisonError};

use super::hit::{Line, Placed, Rect, Zone, line_at, target_at};
use super::refs::{CardRef, Target};
use super::targets::DropTargets;
use crate::answer::Reason;

/// Pattern: Strategy - where an element is, in window coordinates; `None` = not laid out.
pub trait Bounds: Send + Sync {
    /// The rectangle of the element `id`.
    fn bounds(&self, id: &str) -> Option<Rect>;
}

/// Blinc's own answer: `selector::query(id).bounds()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BlincBounds;

impl Bounds for BlincBounds {
    fn bounds(&self, id: &str) -> Option<Rect> {
        blinc_layout::selector::query(id)?
            .bounds()
            .map(|b| Rect::new(b.x, b.y, b.width, b.height))
    }
}

/// Pattern: Strategy - how far the container's content is scrolled now: the translation of the content (negative when scrolled
/// down), the same way every time.
pub trait Scrolled: Send + Sync {
    /// The scroll translation now.
    fn offset(&self) -> (f32, f32);
}

type Rule = Arc<dyn Fn(&CardRef, &Target) -> Result<(), Reason> + Send + Sync>;

/// The places of a container: cells and cards by element id, read at every movement.
#[derive(Clone)]
pub struct MeasuredTargets {
    bounds: Arc<dyn Bounds>,
    scroll: Option<Arc<dyn Scrolled>>,
    zones: Vec<(Option<String>, String, String)>,
    cards: Vec<(CardRef, Option<String>, String, String)>,
    rule: Option<Rule>,
    laid_out: Arc<Mutex<(f32, f32)>>,
}

impl MeasuredTargets {
    /// Places read through `bounds`.
    pub fn new(bounds: Arc<dyn Bounds>) -> Self {
        Self {
            bounds,
            scroll: None,
            zones: Vec::new(),
            cards: Vec::new(),
            rule: None,
            laid_out: Arc::new(Mutex::new((0.0, 0.0))),
        }
    }

    /// A cell that takes cards: its lane (if any), column and the id of its element.
    #[must_use]
    pub fn zone(mut self, lane: Option<&str>, column: &str, element: &str) -> Self {
        self.zones.push((
            lane.map(str::to_owned),
            column.to_owned(),
            element.to_owned(),
        ));
        self
    }

    /// A card shown in a cell, in the order it is shown.
    #[must_use]
    pub fn card(mut self, card: CardRef, lane: Option<&str>, column: &str, element: &str) -> Self {
        self.cards.push((
            card,
            lane.map(str::to_owned),
            column.to_owned(),
            element.to_owned(),
        ));
        self
    }

    /// The container scrolls: the rectangles are corrected by the scroll since the last layout.
    #[must_use]
    pub fn scrolled(mut self, scroll: Arc<dyn Scrolled>) -> Self {
        *self.laid_out.lock().unwrap_or_else(PoisonError::into_inner) = scroll.offset();
        self.scroll = Some(scroll);
        self
    }

    /// Which cell takes which card (the container's rule); without it every cell takes every card.
    #[must_use]
    pub fn rule(mut self, rule: Rule) -> Self {
        self.rule = Some(rule);
        self
    }

    /// A layout has run (the container calls this once per laid-out frame): the rectangles are true for the scroll of now.
    pub fn layout_done(&self) {
        if let Some(s) = &self.scroll {
            *self.laid_out.lock().unwrap_or_else(PoisonError::into_inner) = s.offset();
        }
    }

    /// The scroll the last layout counted in is known from outside (the container reads where the content lies in the laid-out tree):
    /// the rectangles are corrected by the scroll since THAT, no matter when the layout ran.
    pub fn laid_out_at(&self, offset: (f32, f32)) {
        *self.laid_out.lock().unwrap_or_else(PoisonError::into_inner) = offset;
    }

    fn shift(&self) -> (f32, f32) {
        let at_layout = *self.laid_out.lock().unwrap_or_else(PoisonError::into_inner);
        self.scroll.as_ref().map_or((0.0, 0.0), |s| {
            let now = s.offset();
            (now.0 - at_layout.0, now.1 - at_layout.1)
        })
    }

    /// The cells and cards as they lie now (shifted by the scroll since the layout); what is not laid out is left out.
    pub fn read(&self) -> (Vec<Zone>, Vec<Placed>) {
        let d = self.shift();
        let zones = self
            .zones
            .iter()
            .filter_map(|(lane, column, id)| {
                Some(Zone {
                    lane: lane.clone(),
                    column: column.clone(),
                    rect: self.bounds.bounds(id)?.shifted(d),
                })
            })
            .collect();
        let cards = self
            .cards
            .iter()
            .filter_map(|(card, lane, column, id)| {
                Some(Placed {
                    card: card.clone(),
                    lane: lane.clone(),
                    column: column.clone(),
                    rect: self.bounds.bounds(id)?.shifted(d),
                })
            })
            .collect();
        (zones, cards)
    }

    /// The line that shows where a card would land at `to`, in window coordinates (the container draws it).
    pub fn line(&self, to: &Target) -> Option<Line> {
        let (zones, cards) = self.read();
        line_at(&zones, &cards, to)
    }
}

impl DropTargets for MeasuredTargets {
    fn target_at(&self, window: (f32, f32), carried: &CardRef) -> Option<Target> {
        let (zones, cards) = self.read();
        target_at(&zones, &cards, window, Some(carried))
    }

    fn can_drop(&self, carried: &CardRef, to: &Target) -> Result<(), Reason> {
        self.rule.as_ref().map_or(Ok(()), |r| r(carried, to))
    }
}
