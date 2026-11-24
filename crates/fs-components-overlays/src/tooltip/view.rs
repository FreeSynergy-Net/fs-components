//! The tooltip as elements and its handle.
//!
//! Pattern: Facade + Observer + Decorator — a clonable handle over the state; every change draws the bubble again through `Live`.
//! A hover starts a short-lived timer thread only when the delay is above zero (the draw then finds the delay over).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{DEFAULT_DELAY_MS, State};
use crate::live::Live;
use crate::toast::now_ms;

/// A tooltip for one trigger. Clones share the state.
#[derive(Clone)]
pub struct Tooltip {
    id: String,
    words: String,
    state: Arc<Mutex<State>>,
    live: Live,
}

impl Tooltip {
    /// A tooltip saying `words`.
    pub fn new(words: &str) -> Self {
        Self {
            id: "tooltip".to_owned(),
            words: words.to_owned(),
            state: Arc::new(Mutex::new(State::new(DEFAULT_DELAY_MS))),
            live: Live::new(),
        }
    }

    /// The prefix of the ids of the parts: the host is `{id}`, the bubble `{id}-bubble`.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The pause before a hover shows the bubble, in milliseconds (0: at once).
    #[must_use]
    pub fn delay(self, ms: u64) -> Self {
        self.state().delay_ms = ms;
        self
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The id of the bubble, as the trigger names it in `aria-describedby`.
    pub fn bubble_id(&self) -> String {
        format!("{}-bubble", self.id)
    }

    /// Whether the bubble shows now.
    pub fn is_visible(&self) -> bool {
        self.is_visible_at(now_ms())
    }

    /// Whether the bubble shows at time `now` (milliseconds).
    pub fn is_visible_at(&self, now: u64) -> bool {
        self.state().visible(now)
    }

    /// The pointer came (`true`) or went, at time `now`.
    pub fn hover_at(&self, on: bool, now: u64) {
        self.state().hover(on, now);
        self.live.refresh_now();
    }

    /// The pointer came (`true`) or went; a coming pointer draws again once the delay is over.
    pub fn hover(&self, on: bool) {
        self.hover_at(on, now_ms());
        let delay = self.state().delay_ms;
        if on && delay > 0 {
            let live = self.live;
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(delay));
                live.refresh_now();
            });
        }
    }

    /// The trigger got (`true`) or lost the keyboard.
    pub fn focus(&self, on: bool) {
        self.state().focus(on);
        self.live.refresh_now();
    }

    /// A press on the trigger: the bubble opens and stays (touch has no hover); Escape closes it.
    pub fn open(&self) {
        self.state().open();
        self.live.refresh_now();
    }

    /// Escape: close the bubble.
    pub fn escape(&self) {
        self.state().escape();
        self.live.refresh_now();
    }

    /// The trigger with its tooltip under it. `trigger` must be a tab stop (a button): it gets `aria-describedby` and the focus
    /// and blur handlers; the host gets the hover handlers and Escape.
    pub fn attach(&self, trigger: Div) -> Div {
        let (enter, leave, on, off, key) = (
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        );
        let trigger = trigger
            .attr("aria-describedby", self.bubble_id())
            .on_focus(move |_| on.focus(true))
            .on_blur(move |_| off.focus(false));
        let this = self.clone();
        div()
            .id(&self.id)
            .class("fs-tooltip-host")
            .on_hover_enter(move |_| enter.hover(true))
            .on_hover_leave(move |_| leave.hover(false))
            .on_key_down(move |ctx| {
                if ctx.key_code == 27 {
                    key.escape();
                }
            })
            .child(trigger)
            .child(self.live.view("tooltip", move || this.bubble()))
    }

    fn bubble(&self) -> Div {
        let slot = div().class("fs-tooltip-slot");
        if !self.is_visible() {
            return slot;
        }
        slot.child(
            div()
                .absolute()
                .id(self.bubble_id())
                .class("fs-tooltip")
                .class("fs-tooltip-open")
                .attr("role", "tooltip")
                .child(text(self.words.clone()).class("fs-tooltip-text")),
        )
    }
}
