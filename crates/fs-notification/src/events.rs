//! What the facade tells: [`Event`], who listens ([`Observer`]) and what is open ([`Open`]).
//!
//! Pattern: Observer (the interface) + Value objects.

use crate::message::{ActionId, NoteId};
use crate::notification::{CloseReason, Notification};
use crate::outlet::Form;

/// What happened to a notification; observers get these.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Shown (or replaced, with the same id) in these forms.
    Shown {
        /// Its id.
        id: NoteId,
        /// The forms it was shown in.
        forms: Vec<Form>,
    },
    /// The person chose an action.
    Answered {
        /// Its id.
        id: NoteId,
        /// The action.
        action: ActionId,
    },
    /// Gone.
    Closed {
        /// Its id.
        id: NoteId,
        /// Why.
        reason: CloseReason,
    },
}

/// Somebody who wants to hear what happens.
pub trait Observer: Send + Sync {
    /// Called after the change, outside every lock: an observer may call the facade again.
    fn on_event(&self, event: &Event);
}

impl<F: Fn(&Event) + Send + Sync> Observer for F {
    fn on_event(&self, event: &Event) {
        self(event);
    }
}

/// An open notification, as the facade keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    /// Its id.
    pub id: NoteId,
    /// What was shown.
    pub notification: Notification,
    /// The forms it was shown in.
    pub forms: Vec<Form>,
    /// When it expires (ms of the clock handed in), or `None` for until it is closed.
    pub deadline_ms: Option<u64>,
}
