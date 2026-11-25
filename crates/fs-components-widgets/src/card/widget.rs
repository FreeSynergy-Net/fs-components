//! The card: one component with a click and a carry, for every program that shows a thing in a frame.
//!
//! Pattern: State (the gesture and the keys, both pure tables) + Observer (what happens goes to [`CardEvents`]). The card does not know
//! what it shows (the caller hands the parts: header, title, body, footer) nor where it falls (a [`DropTargets`] answers). Without
//! targets it is only clickable: a person card, an installer tile.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::gesture::{Gesture, Moved, Release};
use super::keys::{Mode, Step, step};
use super::refs::{CardRef, Carry, Target};
use super::targets::{CardEvents, DropTargets, NoEvents, Via};
use crate::answer::{Answer, Reason};
use crate::keys::KeyEvent;
use crate::sortable_list::Command;

/// What a gesture or a key came to.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Nothing for the card (the key is the container's or the window's).
    Nothing,
    /// The trigger `activate` was told to the owner.
    Activated,
    /// The trigger `menu` was told to the owner.
    Menu,
    /// The card was lifted by the keyboard.
    Lifted,
    /// A held card is to go to the next place in this direction: the container walks and says the place with [`Card::choose`].
    Walk(Command),
    /// Put down; the owner's answer.
    Dropped(Answer),
    /// Given up or put down over no place: the card stays where it was.
    Back,
    /// Not done, and why (a locked card, a place that does not take it).
    Refused(Reason),
}

pub(super) struct Inner {
    gesture: Gesture<CardRef, Target, Target>,
    held: Option<Target>,
    is_held: bool,
    locked: Option<Reason>,
    blocked: Option<Reason>,
}

/// A card. Clones share the state.
#[derive(Clone)]
pub struct Card {
    pub(super) id: String,
    pub(super) key: CardRef,
    pub(super) at: Target,
    pub(super) panel: bool,
    pub(super) events: Arc<dyn CardEvents>,
    pub(super) targets: Option<Arc<dyn DropTargets>>,
    pub(super) inner: Arc<Mutex<Inner>>,
}

impl Card {
    /// A card with the element id `id` for the object `key`.
    pub fn new(id: impl Into<String>, key: CardRef) -> Self {
        Self {
            id: id.into(),
            key,
            at: Target::default(),
            panel: false,
            events: Arc::new(NoEvents),
            targets: None,
            inner: Arc::new(Mutex::new(Inner {
                gesture: Gesture::Idle,
                held: None,
                is_held: false,
                locked: None,
                blocked: None,
            })),
        }
    }

    /// Where the card stands now (the origin of a carry).
    #[must_use]
    pub fn at(mut self, at: Target) -> Self {
        self.at = at;
        self
    }

    /// The variant `panel` (roomier, for a framed area that holds one thing).
    #[must_use]
    pub fn panel(mut self) -> Self {
        self.panel = true;
        self
    }

    /// Who is told what happens.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn CardEvents>) -> Self {
        self.events = events;
        self
    }

    /// Where the card may be carried; without it the card is only clickable.
    #[must_use]
    pub fn targets(mut self, targets: Arc<dyn DropTargets>) -> Self {
        self.targets = Some(targets);
        self
    }

    /// The card may not be lifted: the reason is said when somebody tries.
    #[must_use]
    pub fn locked(self, reason: Reason) -> Self {
        self.lock().locked = Some(reason);
        self
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The object the card stands for.
    pub fn key(&self) -> &CardRef {
        &self.key
    }

    /// How the card stands.
    pub fn mode(&self) -> Mode {
        let s = self.lock();
        if s.locked.is_some() {
            Mode::Locked
        } else if s.gesture.carried().is_some() {
            Mode::Carried
        } else if s.is_held {
            Mode::Held
        } else {
            Mode::Rest
        }
    }

    /// Why the place under the pointer (or the card itself) does not take it; shown at once.
    pub fn blocked(&self) -> Option<Reason> {
        self.lock().blocked.clone()
    }

    /// While carried: the place it would land at and the pointer. While held: the place chosen and `(0, 0)`.
    pub fn over(&self) -> Option<(Option<Target>, (f32, f32))> {
        let s = self.lock();
        match &s.gesture {
            Gesture::Dragging { over, pointer, .. } => Some((over.clone(), *pointer)),
            _ if s.is_held => Some((s.held.clone(), (0.0, 0.0))),
            _ => None,
        }
    }

    // ---- the pointer -----------------------------------------------------------------------------------------------------

    /// The button went down at `window` (window coordinates).
    pub fn press(&self, window: (f32, f32)) {
        let mut s = self.lock();
        s.blocked = None;
        s.gesture.press(self.key.clone(), self.at.clone(), window);
    }

    /// The pointer moved to `window`. A press becomes a carry after 4 px, never on a locked card (the reason is said instead) and
    /// never without targets.
    pub fn pointer(&self, window: (f32, f32)) -> Moved {
        let Some(targets) = &self.targets else {
            return Moved::Nothing;
        };
        let mut s = self.lock();
        if let Some(reason) = s.locked.clone() {
            if let Gesture::Pressed { at, .. } = &s.gesture
                && (window.0 - at.0).hypot(window.1 - at.1) >= super::gesture::THRESHOLD
            {
                s.gesture.cancel();
                s.blocked = Some(reason);
            }
            return Moved::Nothing;
        }
        let moved = s.gesture.pointer(window, window, &Via(targets.as_ref()));
        let blocked = match &s.gesture {
            Gesture::Dragging { over: Some(to), .. } => targets.can_drop(&self.key, to).err(),
            _ => None,
        };
        s.blocked = blocked;
        drop(s);
        if moved == Moved::Started {
            self.events.changed(&self.key);
        }
        moved
    }

    /// The button went up (also called by the click that follows a drag: a second call finds nothing going on).
    pub fn release(&self) -> Outcome {
        let (done, blocked) = {
            let mut s = self.lock();
            (s.gesture.release(), s.blocked.take())
        };
        match done {
            Release::Nothing => Outcome::Nothing,
            Release::Click(_) => {
                self.events.activate(&self.key);
                Outcome::Activated
            }
            Release::Back => {
                self.events.changed(&self.key);
                Outcome::Back
            }
            Release::Drop { card, from, spot } => {
                self.events.changed(&self.key);
                if let Some(reason) = blocked {
                    return Outcome::Refused(reason);
                }
                Outcome::Dropped(self.events.dropped(&Carry { key: card, from }, &spot))
            }
        }
    }

    // ---- the keyboard ----------------------------------------------------------------------------------------------------

    /// A key. The table is `keys.rs`; `Outcome::Nothing` = not the card's.
    pub fn key_press(&self, event: &KeyEvent) -> Outcome {
        let mode = self.mode();
        match step(mode, event) {
            Step::Pass => Outcome::Nothing,
            Step::Activate => {
                self.events.activate(&self.key);
                Outcome::Activated
            }
            Step::Menu => {
                self.events.menu(&self.key, self.anchor());
                Outcome::Menu
            }
            Step::Refuse => {
                let reason = self.lock().locked.clone();
                reason.map_or(Outcome::Nothing, |r| {
                    self.lock().blocked = Some(r.clone());
                    Outcome::Refused(r)
                })
            }
            Step::Sort(Command::Toggle) => self.toggle(),
            Step::Sort(Command::Cancel) => {
                let mut s = self.lock();
                let was = s.is_held || s.gesture.cancel();
                s.is_held = false;
                s.held = None;
                s.blocked = None;
                drop(s);
                self.events.changed(&self.key);
                if was { Outcome::Back } else { Outcome::Nothing }
            }
            Step::Sort(walk) => Outcome::Walk(walk),
        }
    }

    fn toggle(&self) -> Outcome {
        let to = {
            let mut s = self.lock();
            if !s.is_held {
                s.is_held = true;
                s.held = Some(self.at.clone());
                drop(s);
                self.events.changed(&self.key);
                return Outcome::Lifted;
            }
            s.is_held = false;
            s.held.take()
        };
        self.events.changed(&self.key);
        let Some(to) = to else { return Outcome::Back };
        if let Some(Err(reason)) = self.targets.as_ref().map(|t| t.can_drop(&self.key, &to)) {
            self.lock().blocked = Some(reason.clone());
            return Outcome::Refused(reason);
        }
        let carry = Carry {
            key: self.key.clone(),
            from: self.at.clone(),
        };
        Outcome::Dropped(self.events.dropped(&carry, &to))
    }

    /// The container walked a held card to `to`: the place is shown, the reason if the place does not take it.
    pub fn choose(&self, to: Target) {
        let blocked = self
            .targets
            .as_ref()
            .and_then(|t| t.can_drop(&self.key, &to).err());
        let mut s = self.lock();
        if s.is_held {
            s.held = Some(to);
            s.blocked = blocked;
        }
    }

    /// Where a menu opens: under the card's left edge, from its measured place; `(0, 0)` while it is not laid out.
    fn anchor(&self) -> (f32, f32) {
        blinc_layout::selector::query(&self.id)
            .and_then(|q| q.bounds())
            .map_or((0.0, 0.0), |b| (b.x, b.y + b.height))
    }
}
