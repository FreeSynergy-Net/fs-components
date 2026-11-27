//! The places a card can land on the board, and what the card component tells the board.
//!
//! Pattern: Adapter (the cells and cards of the board in, `MeasuredTargets` and `CardEvents` of the card component out). The board has
//! no rectangle of its own: a cell is the element `<id>-col-<column>` (`<id>-lane-<lane>-col-<column>` in a lane), a card the element
//! of its card; where they are is read from the laid-out tree while a card is carried. The card component knows no board, only a
//! `CardRef` (a reference to an object) and a `Target` (lane, column, the neighbours); the key of a card (an opaque text of the
//! program) is put into an IRI `urn:fs-board:<board>:<key>` and taken out again here, nowhere else.

use std::sync::{Arc, PoisonError};

use fs_components_widgets::answer::{Answer, Reason};
use fs_components_widgets::card::{
    BlincBounds, CardEvents, CardRef, Carry, MeasuredTargets, Scrolled, Target,
};

use crate::bench::BoardBench;
use crate::state::Action;
use crate::swimlanes::Lane;

/// What a card of the board is (a task is an AS2 object; the board does not look into it).
const MEDIA_TYPE: &str = "application/activity+json";

fn escape(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(b).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn unescape(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(b)) => {
                out.push(b);
                i += 3;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The reference of the card `key` of the board `board`.
pub(crate) fn ref_of(board: &str, key: &str) -> CardRef {
    let iri = format!("urn:fs-board:{}:{}", escape(board), escape(key));
    CardRef::parse(&iri, MEDIA_TYPE).unwrap_or_else(|_| {
        CardRef::parse("urn:fs-board:none:none", MEDIA_TYPE)
            .unwrap_or_else(|_| unreachable!("a constant absolute IRI"))
    })
}

/// The key of the card a reference stands for (the part after the last colon of the IRI).
pub(crate) fn key_of(card: &CardRef) -> String {
    let iri = card.iri_text();
    unescape(iri.rsplit(':').next().unwrap_or_default())
}

/// How far the scroll area is scrolled, as the translation of its content (what `MeasuredTargets` corrects by).
pub(crate) struct ScrollOffset(pub(crate) blinc_layout::selector::ScrollRef);

impl Scrolled for ScrollOffset {
    fn offset(&self) -> (f32, f32) {
        let (x, y) = self.0.offset();
        // Blinc's sign is not relied on: the content moves up and to the left when scrolled.
        (-x.abs(), -y.abs())
    }
}

/// What the cards tell the board.
pub(crate) struct Events(pub(crate) BoardBench);

impl CardEvents for Events {
    fn activate(&self, card: &CardRef) {
        self.0.sync_rights();
        if !self.0.read(|s| s.can_move) {
            self.0.say_move_soon();
        }
        self.0.card_click(&key_of(card));
    }

    fn menu(&self, card: &CardRef, at: (f32, f32)) {
        self.0
            .context(&crate::ContextTarget::Card(key_of(card)), at);
    }

    fn dropped(&self, carry: &Carry, to: &Target) -> Answer {
        self.0.drop_at(carry, to)
    }

    fn cancelled(&self, _card: &CardRef) {
        *self
            .0
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }

    fn changed(&self, card: &CardRef) {
        self.0.carried_changed(card);
    }
}

impl BoardBench {
    /// The cells and the cards of the board as measured places. Built from the data (which cells and cards there are), read from the
    /// tree when a card is carried; what is folded away (a closed lane) is not laid out and so not a place.
    pub(crate) fn places(
        &self,
        state: &crate::BoardState,
        lanes: Option<&[Lane]>,
    ) -> Arc<MeasuredTargets> {
        let (source, soon) = (self.source.clone(), self.labels.move_soon.clone());
        let rule = move |card: &CardRef, to: &Target| {
            // Without the right to move every place says so (the card is still clickable); else the program decides.
            if !source.supports(crate::Operation::Move) {
                return Err(Reason::new("unsupported", &soon));
            }
            source.can_drop(&key_of(card), &to.column, to.lane.as_deref())
        };
        let mut places = MeasuredTargets::new(Arc::new(BlincBounds))
            .scrolled(Arc::new(ScrollOffset(self.scroll.clone())))
            .rule(Arc::new(rule));
        let id = &self.id;
        for column in state.lanes() {
            let col = &column.key;
            match lanes {
                None => {
                    places = places.zone(None, col, &format!("{id}-col-{col}"));
                    for item in &column.items {
                        let element = format!("{id}-card-{}", item.key);
                        places = places.card(ref_of(id, &item.key), None, col, &element);
                    }
                }
                Some(lanes) => {
                    for lane in lanes {
                        let l = &lane.key;
                        places = places.zone(Some(l), col, &format!("{id}-lane-{l}-col-{col}"));
                        for item in column.items.iter().filter(|i| lane.cards.contains(&i.key)) {
                            let element = format!("{id}-lane-{l}-card-{}", item.key);
                            places = places.card(ref_of(id, &item.key), Some(l), col, &element);
                        }
                    }
                }
            }
        }
        let places = Arc::new(places);
        *self.places.lock().unwrap_or_else(PoisonError::into_inner) = Some(places.clone());
        places
    }

    /// A card was put down at `to`: the same road as every move (`place_in`), the program's answer.
    fn drop_at(&self, carry: &Carry, to: &Target) -> Answer {
        self.sync_rights();
        let (key, after, before) = (
            key_of(&carry.key),
            to.after.as_ref().map(key_of),
            to.before.as_ref().map(key_of),
        );
        let action = {
            let mut state = self.lock();
            let index = state.index_at(&to.column, &key, after.as_deref(), before.as_deref());
            let lanes = (carry.from.lane.clone(), to.lane.clone());
            state.place_in(&key, &to.column, index, lanes, self.source.as_ref(), false)
        };
        match action {
            Action::Move(request, undo) => self.move_to(&request, Some(undo)),
            other => {
                self.perform(other);
                Answer::Accepted
            }
        }
    }

    /// The carried card moved or changed its state: the layer (the line and the ghost) is drawn again, near an edge the board scrolls,
    /// a locked card that was tried to lift says why, and a card let go over a place that refuses leaves the reason as a notice.
    fn carried_changed(&self, card: &CardRef) {
        self.layout_seen();
        let found = self
            .cards
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|(_, c)| c.key() == card)
            .map(|(id, c)| (id.clone(), c.clone()))
            .find(|(_, c)| c.over().is_some() || c.blocked().is_some());
        let mut active = self.active.lock().unwrap_or_else(PoisonError::into_inner);
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let Some((id, card)) = found else {
            *active = None;
            let refused = pending.take();
            drop((active, pending));
            self.drag_live.refresh();
            if let Some(reason) = refused {
                self.lock().refuse(&reason);
                self.refresh();
            }
            return;
        };
        let Some((_, pointer)) = card.over() else {
            *pending = None;
            drop((active, pending));
            if let Some(reason) = card.blocked() {
                self.lock().refuse(&reason);
                self.refresh();
            }
            return;
        };
        *pending = card.blocked();
        *active = Some((id, card));
        drop((active, pending));
        let step = self.scroll_step(pointer);
        if step != (0.0, 0.0) {
            self.scroll.scroll_by(step.0, step.1);
        }
        self.drag_live.refresh();
    }

    /// Escape while a card is carried by the pointer gives the carry up (the cards are no tab stops, so the grid hands the key on).
    pub(crate) fn cancel_carry(&self, event: &fs_components_widgets::keys::KeyEvent) -> bool {
        let active = self
            .active
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        active.is_some_and(|(_, card)| {
            card.over().is_some()
                && card.key_press(event) != fs_components_widgets::card::Outcome::Nothing
        })
    }

    /// The scroll the last layout counted in, READ from the laid-out tree: where the content lies against the scroll box (the registry
    /// holds the positions of the last layout, a scroll alone does not move them). Nothing is guessed, no hook is needed.
    fn layout_seen(&self) {
        let at = |what: &str| {
            blinc_layout::selector::query(&format!("{}-{what}", self.id)).and_then(|q| q.bounds())
        };
        if let (Some(content), Some(frame), Some(places)) =
            (at("content"), at("scrollbox"), self.current_places())
        {
            places.laid_out_at((content.x - frame.x, content.y - frame.y));
        }
    }

    /// Near the edge of the viewport the board scrolls: the step for a pointer at `window` (window coordinates).
    fn scroll_step(&self, window: (f32, f32)) -> (f32, f32) {
        let (Some((w, h)), Some(origin)) = (
            self.scroll.viewport_size(),
            blinc_layout::selector::query(&format!("{}-scrollbox", self.id))
                .and_then(|q| q.bounds()),
        ) else {
            return (0.0, 0.0);
        };
        (
            crate::scroll::step(window.0 - origin.x, 0.0, w),
            crate::scroll::step(window.1 - origin.y, 0.0, h),
        )
    }
}

impl BoardBench {
    /// The places of the board as drawn last (`None` before the first draw).
    pub(crate) fn current_places(&self) -> Option<Arc<MeasuredTargets>> {
        self.places
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
