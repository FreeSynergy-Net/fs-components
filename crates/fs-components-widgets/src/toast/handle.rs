//! The toast stack a program keeps.
//!
//! Pattern: Facade + Observer — a clonable handle over the pure stack; every change draws the stack again through `Live`.
//! The frontend calls [`Toasts::tick_now`] on a timer (once a second is enough) and puts [`Toasts::build`] in a corner of its
//! window; nothing else is needed. Time is explicit in the `_at` calls, so a test needs no waiting.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use super::labels::Labels;
use super::model::{FOCUS, POINTER, Stack, Times, Toast};
use crate::Element;
use crate::live::Live;

/// Milliseconds since the first call in this process: the clock of the `_now` calls.
pub fn now_ms() -> u64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    u64::try_from(START.get_or_init(Instant::now).elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// The stack of toasts. Clones share it.
#[derive(Clone)]
pub struct Toasts {
    pub(super) id: String,
    pub(super) labels: Labels,
    stack: Arc<Mutex<Stack>>,
    live: Live,
}

impl Toasts {
    /// An empty stack.
    pub fn new(labels: &Labels) -> Self {
        Self {
            id: "toasts".to_owned(),
            labels: labels.clone(),
            stack: Arc::new(Mutex::new(Stack::new())),
            live: Live::new(),
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// How many toasts are in view at once (at least one); older ones wait behind a count.
    #[must_use]
    pub fn max_visible(self, n: usize) -> Self {
        self.stack().max = n.max(1);
        self
    }

    /// How long toasts of each kind stay (a setting; see [`Times`]). Toasts already on the stack keep their time.
    #[must_use]
    pub fn times(self, times: Times) -> Self {
        self.stack().times = times;
        self
    }

    pub(super) fn stack(&self) -> MutexGuard<'_, Stack> {
        self.stack.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Show `toast` now; its id.
    pub fn push(&self, toast: Toast) -> u64 {
        self.push_at(toast, now_ms())
    }

    /// Show `toast` at time `now` (milliseconds); its id.
    pub fn push_at(&self, toast: Toast, now: u64) -> u64 {
        let id = self.stack().push(toast, now);
        self.live.refresh_now();
        id
    }

    /// Take away what has run out.
    pub fn tick_now(&self) -> usize {
        self.tick(now_ms())
    }

    /// Take away what has run out at time `now`; how many went.
    pub fn tick(&self, now: u64) -> usize {
        let gone = self.stack().tick(now);
        if gone > 0 {
            self.live.refresh_now();
        }
        gone
    }

    /// Close one toast; false when it is not there (any more).
    pub fn dismiss(&self, id: u64) -> bool {
        let gone = self.stack().dismiss(id);
        if gone {
            self.live.refresh_now();
        }
        gone
    }

    /// Close all.
    pub fn clear(&self) {
        self.stack().clear();
        self.live.refresh_now();
    }

    /// Hold a toast (the pointer is over it): its time stands still until released.
    pub fn hold(&self, id: u64, held: bool) {
        self.hold_at(id, held, now_ms());
    }

    /// [`Toasts::hold`] at time `now`.
    pub fn hold_at(&self, id: u64, held: bool, now: u64) {
        self.stack().hold(id, POINTER, held, now);
    }

    /// The keyboard is on a toast (`true`) or has left it: its time stands still while it is on it, whatever the pointer does.
    pub fn hold_keyboard(&self, id: u64, held: bool) {
        self.hold_keyboard_at(id, held, now_ms());
    }

    /// [`Toasts::hold_keyboard`] at time `now`.
    pub fn hold_keyboard_at(&self, id: u64, held: bool, now: u64) {
        self.stack().hold(id, FOCUS, held, now);
    }

    /// Run the action of a toast and close it; false when it has none or is gone.
    pub fn run_action(&self, id: u64) -> bool {
        let act = self.stack().action_of(id);
        let Some(act) = act else {
            return false;
        };
        act();
        self.dismiss(id);
        true
    }

    /// How many toasts there are, in view or waiting.
    pub fn count(&self) -> usize {
        self.stack().len()
    }

    /// The element to put in a corner of the window.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.region("toasts", move || this.render())
    }
}
