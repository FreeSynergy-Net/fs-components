//! The pure state of the toast stack: what is shown, for how long, and when it goes.
//!
//! Pattern: State + Strategy for time (every call that depends on time takes `now` in milliseconds, so a test needs no waiting
//! and the frontend decides where time comes from). No Blinc here.

use std::sync::Arc;

use crate::states::Severity;

/// What a button of a toast does.
pub type Act = Arc<dyn Fn() + Send + Sync>;

/// How many toasts are shown at once, unless told otherwise; older ones wait behind a count.
pub const MAX_VISIBLE: usize = 4;

/// One short message.
#[derive(Clone)]
pub struct Toast {
    pub(super) kind: Severity,
    pub(super) text: String,
    pub(super) action: Option<(String, Act)>,
    pub(super) lasts_ms: Option<u64>,
}

impl Toast {
    /// A toast of `kind` that goes by itself after the time of its kind (`Severity::Error` stays until it is closed: a failure
    /// must not vanish before it was read).
    pub fn new(kind: Severity, text: &str) -> Self {
        let lasts_ms = match kind {
            Severity::Info => Some(5_000),
            Severity::Ok => Some(4_000),
            Severity::Warn => Some(8_000),
            Severity::Error => None,
        };
        Self {
            kind,
            text: text.to_owned(),
            action: None,
            lasts_ms,
        }
    }

    /// A button with `label` ("Undo") that runs `handler` and closes the toast.
    #[must_use]
    pub fn action(mut self, label: &str, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.action = Some((label.to_owned(), Arc::new(handler)));
        self
    }

    /// Goes after `ms` milliseconds.
    #[must_use]
    pub fn lasts(mut self, ms: u64) -> Self {
        self.lasts_ms = Some(ms);
        self
    }

    /// Stays until it is closed.
    #[must_use]
    pub fn stays(mut self) -> Self {
        self.lasts_ms = None;
        self
    }
}

/// A toast on the stack.
pub(super) struct Item {
    pub id: u64,
    pub toast: Toast,
    /// When it goes (`None`: stays, or is held).
    expires: Option<u64>,
    /// The time left while it is held (the pointer is over it or it has the keyboard).
    left: Option<u64>,
}

/// The stack: the newest toast is last.
pub(super) struct Stack {
    items: Vec<Item>,
    next: u64,
    pub max: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next: 1,
            max: MAX_VISIBLE,
        }
    }

    pub fn push(&mut self, toast: Toast, now: u64) -> u64 {
        let id = self.next;
        self.next += 1;
        let expires = toast.lasts_ms.map(|ms| now.saturating_add(ms));
        self.items.push(Item {
            id,
            toast,
            expires,
            left: None,
        });
        id
    }

    /// Drop what has run out; how many went.
    pub fn tick(&mut self, now: u64) -> usize {
        let before = self.items.len();
        self.items
            .retain(|i| i.left.is_some() || i.expires.is_none_or(|t| t > now));
        before - self.items.len()
    }

    pub fn dismiss(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id != id);
        before != self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Hold (`true`) or release a toast: while held its time stands still.
    pub fn hold(&mut self, id: u64, held: bool, now: u64) {
        let Some(item) = self.items.iter_mut().find(|i| i.id == id) else {
            return;
        };
        if held && item.left.is_none() {
            item.left = item.expires.map(|t| t.saturating_sub(now));
            item.expires = None;
        } else if !held && let Some(left) = item.left.take() {
            item.expires = Some(now.saturating_add(left));
        }
    }

    pub fn action_of(&self, id: u64) -> Option<Act> {
        let item = self.items.iter().find(|i| i.id == id)?;
        item.toast.action.as_ref().map(|(_, act)| Arc::clone(act))
    }

    /// The toasts in view, oldest first.
    pub fn shown(&self) -> &[Item] {
        &self.items[self.items.len().saturating_sub(self.max)..]
    }

    /// How many older toasts wait behind the ones in view.
    pub fn hidden(&self) -> usize {
        self.items.len().saturating_sub(self.max)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
