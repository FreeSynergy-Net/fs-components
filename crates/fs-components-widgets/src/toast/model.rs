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
    /// Whether the toast named its own time (`lasts`, `stays`); else the time of its kind in [`Times`] counts.
    pub(super) chosen: bool,
}

/// How long a toast of each kind stays, in milliseconds (`None`: until it is closed). A setting of the stack
/// ([`super::Toasts::times`]), not a fixed number: WAI-ARIA APG advises that an alert does not vanish too fast (WCAG 2.2.3), so a
/// program or a person may lengthen or switch off the going by itself. The default: what has role `alert` (a warning, an error)
/// stays until it is closed, what has role `status` (info, ok) goes by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Times {
    /// Time of an `info` toast.
    pub info: Option<u64>,
    /// Time of an `ok` toast.
    pub ok: Option<u64>,
    /// Time of a `warning` toast.
    pub warning: Option<u64>,
    /// Time of an `error` toast.
    pub error: Option<u64>,
}

impl Default for Times {
    fn default() -> Self {
        Self {
            info: Some(5_000),
            ok: Some(4_000),
            warning: None,
            error: None,
        }
    }
}

impl Times {
    /// Every toast stays until it is closed.
    pub const STAY: Self = Self {
        info: None,
        ok: None,
        warning: None,
        error: None,
    };

    /// The time of a toast of `kind`.
    pub const fn of(&self, kind: Severity) -> Option<u64> {
        match kind {
            Severity::Info => self.info,
            Severity::Ok => self.ok,
            Severity::Warn => self.warning,
            Severity::Error => self.error,
        }
    }
}

impl Toast {
    /// A toast of `kind` that goes by itself after the time of its kind (`Severity::Error` stays until it is closed: a failure
    /// must not vanish before it was read).
    pub fn new(kind: Severity, text: &str) -> Self {
        Self {
            kind,
            text: text.to_owned(),
            action: None,
            lasts_ms: None,
            chosen: false,
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
        self.chosen = true;
        self
    }

    /// Stays until it is closed.
    #[must_use]
    pub fn stays(mut self) -> Self {
        self.lasts_ms = None;
        self.chosen = true;
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
    /// Who holds it: [`POINTER`] and [`FOCUS`] bits. It is released only when BOTH have let go (a pointer leaving a toast whose
    /// close mark still has the keyboard must not start the clock).
    holds: u8,
}

/// The pointer is over a toast.
pub(super) const POINTER: u8 = 1;
/// The keyboard is on a toast.
pub(super) const FOCUS: u8 = 2;

/// The stack: the newest toast is last.
pub(super) struct Stack {
    items: Vec<Item>,
    next: u64,
    pub max: usize,
    pub times: Times,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next: 1,
            max: MAX_VISIBLE,
            times: Times::default(),
        }
    }

    pub fn push(&mut self, toast: Toast, now: u64) -> u64 {
        let id = self.next;
        self.next += 1;
        let lasts = if toast.chosen {
            toast.lasts_ms
        } else {
            self.times.of(toast.kind)
        };
        let expires = lasts.map(|ms| now.saturating_add(ms));
        self.items.push(Item {
            id,
            toast,
            expires,
            left: None,
            holds: 0,
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
    /// `source` ([`POINTER`] or [`FOCUS`]) holds (`true`) or lets go of a toast: while any source holds it, its time stands still.
    pub fn hold(&mut self, id: u64, source: u8, held: bool, now: u64) {
        let Some(item) = self.items.iter_mut().find(|i| i.id == id) else {
            return;
        };
        let before = item.holds != 0;
        if held {
            item.holds |= source;
        } else {
            item.holds &= !source;
        }
        let after = item.holds != 0;
        if after && !before {
            item.left = item.expires.map(|t| t.saturating_sub(now));
            item.expires = None;
        } else if !after
            && before
            && let Some(left) = item.left.take()
        {
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
