//! The carousel as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the slide and the position again through
//! `Live`. The root (keys) and the two buttons stay, so the keyboard stays.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::event_router::keyboard_focused_node;
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::model::{Outcome, State};
use crate::Element;
use crate::keys::{Key, KeyEvent, bind_keys};
use crate::live::Live;

type Slide = Arc<dyn Fn() -> Div + Send + Sync>;
type OnShow = Arc<dyn Fn(usize) + Send + Sync>;

/// A carousel. Clones share the state and the callback.
#[derive(Clone)]
pub struct Carousel {
    id: String,
    name: String,
    labels: Labels,
    slides: Arc<Vec<Slide>>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_show: Option<OnShow>,
}

impl Carousel {
    /// A carousel named `name` ("Featured programs"), the ends joined. Ids: `<id>`, `<id>-previous`, `<id>-next`, `<id>-slide`.
    pub fn new(name: &str, labels: &Labels) -> Self {
        Self {
            id: "carousel".to_owned(),
            name: name.to_owned(),
            labels: labels.clone(),
            slides: Arc::new(Vec::new()),
            state: Arc::new(Mutex::new(State::new(0, true))),
            live: Live::new(),
            on_show: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Add a slide; `content` is built each time it is shown.
    #[must_use]
    pub fn slide(mut self, content: impl Fn() -> Div + Send + Sync + 'static) -> Self {
        let mut slides = (*self.slides).clone();
        slides.push(Arc::new(content));
        let wrap = self.slot().wraps();
        self.state = Arc::new(Mutex::new(State::new(slides.len(), wrap)));
        self.slides = Arc::new(slides);
        self
    }

    /// The first slide does not follow the last and the other way round.
    #[must_use]
    pub fn no_wrap(self) -> Self {
        let count = self.slot().count();
        *self.slot() = State::new(count, false);
        self
    }

    /// Call `handler` with the index of the slide shown after a change by the person.
    #[must_use]
    pub fn on_show(mut self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_show = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The index of the slide shown.
    pub fn current(&self) -> usize {
        self.slot().current()
    }

    /// Show a slide from the program (no callback).
    pub fn go(&self, to: usize) {
        if self.slot().go(to) {
            self.live.refresh_now();
        }
    }

    fn said(&self, outcome: Outcome) -> bool {
        let Outcome::Shown(at) = outcome else {
            return false;
        };
        self.live.refresh_now();
        if let Some(handler) = &self.on_show {
            handler(at);
        }
        true
    }

    /// The next slide.
    pub fn next(&self) -> bool {
        let o = self.slot().forward();
        self.said(o)
    }

    /// The previous slide.
    pub fn previous(&self) -> bool {
        let o = self.slot().back();
        self.said(o)
    }

    /// A press of the keyboard on the carousel.
    pub fn press(&self, event: KeyEvent) -> bool {
        let o = self.slot().press(event.key);
        self.said(o)
    }

    /// The carousel, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (keys, inner, shown) = (self.clone(), self.clone(), self.clone());
        let root = bind_keys(
            div()
                .id(&self.id)
                .class("fs-carousel")
                .attr("role", "region")
                .attr("aria-roledescription", &self.labels.kind)
                .attr("aria-label", &self.name),
            Arc::new(move |event: KeyEvent| {
                // Enter and Space do nothing here; the buttons are for the pointer and their own keys
                !matches!(event.key, Key::Enter | Key::Space | Key::Activate) && keys.press(event)
            }),
        );
        let bar = div()
            .class("fs-carousel-bar")
            .child(self.button("previous", &self.labels.previous, false))
            .child(self.live.region("position", move || {
                let s = shown.slot();
                div()
                    .class("fs-carousel-position")
                    .attr("aria-hidden", "true")
                    .child(text(format!("{} / {}", s.current() + 1, s.count())))
            }))
            .child(self.button("next", &self.labels.next, true));
        root.child(bar)
            .child(self.live.region("slide", move || inner.slide_view()))
    }

    fn button(&self, id: &str, name: &str, forward: bool) -> Div {
        let this = self.clone();
        let glyph = if forward { "\u{25B8}" } else { "\u{25C2}" };
        bind_keys(
            div()
                .id(format!("{}-{id}", self.id))
                .class("fs-carousel-button")
                .attr("role", "button")
                .attr("aria-label", name)
                .child(text(glyph).class("fs-carousel-button-text")),
            {
                let this = this.clone();
                Arc::new(move |event| match event.key {
                    Key::Activate | Key::Enter | Key::Space => {
                        if forward {
                            this.next()
                        } else {
                            this.previous()
                        };
                        true
                    }
                    _ => false,
                })
            },
        )
        .on_click(move |ctx| {
            if keyboard_focused_node() != Some(ctx.node_id) {
                if forward {
                    this.next()
                } else {
                    this.previous()
                };
            }
        })
    }

    fn slide_view(&self) -> Div {
        let (at, count) = {
            let s = self.slot();
            (s.current(), s.count())
        };
        let mut slide = div()
            .id(format!("{}-slide", self.id))
            .class("fs-carousel-slide")
            .attr("role", "group")
            .attr("aria-roledescription", &self.labels.slide)
            .attr("aria-label", format!("{} / {count}", at + 1))
            .attr("aria-live", "polite");
        if let Some(content) = self.slides.get(at) {
            slide = slide.child(content());
        }
        slide
    }
}
