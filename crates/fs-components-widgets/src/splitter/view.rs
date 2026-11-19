//! The splitter as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; every change draws the panes again through `Live`. The
//! divider element carries the pointer handlers and stays while the panes are drawn again.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};

use super::model::{Outcome, State};
use crate::Element;
use crate::keys::{KeyEvent, bind_keys};
use crate::live::Live;
use crate::slider::number;

type OnMove = Arc<dyn Fn(f64) + Send + Sync>;
type Pane = Arc<dyn Fn() -> Div + Send + Sync>;

/// How thick the divider is, in pixels.
pub const BAR: f32 = 8.0;

/// A splitter. Clones share the state and the callback.
#[derive(Clone)]
pub struct Splitter {
    id: String,
    name: String,
    state: Arc<Mutex<State>>,
    start: Arc<Mutex<(f64, f32, f32)>>,
    held: Arc<Mutex<bool>>,
    live: Live,
    on_move: Option<OnMove>,
}

impl Splitter {
    /// A divider named `name` ("Sidebar width") at `position` percent, between `min` and `max`, moving by `step`, between left and
    /// right panes. Ids: `<id>` (the divider), `<id>-first`, `<id>-second`.
    pub fn new(name: &str, position: f64, min: f64, max: f64, step: f64) -> Self {
        Self::with(name, State::new(position, min, max, step, true))
    }

    /// Like [`Splitter::new`] for panes one above the other (Up and Down move the divider).
    pub fn stacked(name: &str, position: f64, min: f64, max: f64, step: f64) -> Self {
        Self::with(name, State::new(position, min, max, step, false))
    }

    fn with(name: &str, state: State) -> Self {
        Self {
            id: "splitter".to_owned(),
            name: name.to_owned(),
            start: Arc::new(Mutex::new((state.position(), 0.0, 0.0))),
            state: Arc::new(Mutex::new(state)),
            held: Arc::new(Mutex::new(false)),
            live: Live::new(),
            on_move: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Call `handler` with the first pane's new size in percent after every change by the person.
    #[must_use]
    pub fn on_move(mut self, handler: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_move = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The first pane's size in percent.
    pub fn position(&self) -> f64 {
        self.slot().position()
    }

    /// Whether the first pane is collapsed.
    pub fn collapsed(&self) -> bool {
        self.slot().collapsed()
    }

    /// Put the divider at `percent` from the program (held to the limits; no callback).
    pub fn set(&self, percent: f64) {
        if self.slot().set(percent) {
            self.live.refresh_now();
        }
    }

    fn said(&self, to: f64) {
        self.live.refresh_now();
        if let Some(handler) = &self.on_move {
            handler(to);
        }
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> bool {
        let outcome = self.slot().press(event.key);
        match outcome {
            Outcome::Ignored => false,
            Outcome::Moved(to) => {
                self.said(to);
                true
            }
        }
    }

    fn drag_to(&self, percent: f64) {
        let moved = self.slot().set(percent);
        if moved {
            let to = self.position();
            self.said(to);
        }
    }

    /// The splitter, `size` pixels along its axis (the width, or the height of stacked panes), the other side `across` pixels.
    pub fn build<A, B>(
        &self,
        size: f32,
        across: f32,
        first: A,
        second: B,
    ) -> impl Element + use<A, B>
    where
        A: Fn() -> Div + Send + Sync + 'static,
        B: Fn() -> Div + Send + Sync + 'static,
    {
        let vertical = self.slot().vertical();
        let run = (size - BAR).max(1.0);
        let (first, second): (Pane, Pane) = (Arc::new(first), Arc::new(second));
        let (down, drag, up) = (self.clone(), self.clone(), self.clone());
        let (one, two, bar) = (self.clone(), self.clone(), self.clone());
        // The grip carries the pointer and stays; the separator inside it carries the ARIA state and the keys and is drawn again.
        let grip = div()
            .id(format!("{}-grip", self.id))
            .on_mouse_down(move |ctx| {
                *down.start.lock().unwrap_or_else(PoisonError::into_inner) =
                    (down.position(), ctx.mouse_x, ctx.mouse_y);
                down.hold(true);
            })
            .on_drag(move |ctx| {
                // from where the press was, by the pointer's own position (the drag delta of the event is not relied on)
                let (from, x, y) = *drag.start.lock().unwrap_or_else(PoisonError::into_inner);
                let delta = if vertical {
                    ctx.mouse_x - x
                } else {
                    ctx.mouse_y - y
                };
                drag.drag_to(from + f64::from(delta / run) * 100.0);
            })
            .on_mouse_up(move |_| up.hold(false))
            .child(
                self.live
                    .region("bar", move || bar.separator(vertical, across)),
            );
        let grip = if vertical {
            grip.w(BAR).h(across)
        } else {
            grip.h(BAR).w(across)
        };
        let root = div().class("fs-splitter");
        let root = if vertical {
            root.flex_row()
        } else {
            root.flex_col().class("fs-splitter-horizontal")
        };
        root.child(self.live.region("first", move || {
            one.pane(true, vertical, run, across, &first)
        }))
        .child(grip)
        .child(self.live.region("second", move || {
            two.pane(false, vertical, run, across, &second)
        }))
    }

    fn hold(&self, on: bool) {
        *self.held.lock().unwrap_or_else(PoisonError::into_inner) = on;
        self.live.refresh_now();
    }

    /// The divider with the ARIA state of the moment: a focusable `separator`.
    fn separator(&self, vertical: bool, across: f32) -> Div {
        let state = self.slot().clone();
        let keys = self.clone();
        let mut bar = bind_keys(
            div()
                .id(format!("{}-bar", self.id))
                .class("fs-splitter-bar")
                .attr("role", "separator")
                .attr("aria-label", &self.name)
                .attr(
                    "aria-orientation",
                    if vertical { "vertical" } else { "horizontal" },
                )
                .attr("aria-controls", format!("{}-first", self.id))
                .attr("aria-valuenow", number(state.position()))
                .attr("aria-valuemin", number(state.min()))
                .attr("aria-valuemax", number(state.max()))
                // the pointer belongs to the grip around it: a press on the divider itself would not be dragged
                .pointer_events_none(),
            Arc::new(move |event| keys.press(event)),
        );
        bar = if vertical {
            bar.w(BAR).h(across)
        } else {
            bar.h(BAR).w(across)
        };
        if *self.held.lock().unwrap_or_else(PoisonError::into_inner) {
            bar = bar.class("fs-splitter-bar-active");
        }
        bar
    }

    fn pane(&self, first: bool, vertical: bool, run: f32, across: f32, content: &Pane) -> Div {
        let percent = self.position();
        #[allow(clippy::cast_possible_truncation)]
        let along = (run * (percent / 100.0) as f32).max(0.0);
        let along = if first { along } else { (run - along).max(0.0) };
        let pane = div()
            .id(format!(
                "{}-{}",
                self.id,
                if first { "first" } else { "second" }
            ))
            .class("fs-splitter-pane");
        let pane = if vertical {
            pane.w(along).h(across)
        } else {
            pane.h(along).w(across)
        };
        pane.child(content())
    }
}
