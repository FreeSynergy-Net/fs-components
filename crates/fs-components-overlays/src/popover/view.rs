//! The popover as elements and its handle.
//!
//! Pattern: Facade + Observer — a clonable handle over the open mark; every change draws the host again through `Live`.
//! The trigger and the content are given as builders, because the host is built again at every change (`aria-expanded` changes).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::{Div, div};

use super::labels::Labels;
use crate::focus::FocusTo;
use crate::icon::{Glyph, Tone, icon_stroked};
use crate::keys::{Key, bind_keys};
use crate::live::Live;
use crate::states::STROKE;

/// What the popover calls when it closed.
type Act = Arc<dyn Fn() + Send + Sync>;

/// A popover. Clones share the open mark and the callbacks.
#[derive(Clone)]
pub struct Popover {
    id: String,
    name: String,
    labels: Labels,
    open: Arc<AtomicBool>,
    live: Live,
    focus_to: Option<FocusTo>,
    on_close: Option<Act>,
}

impl Popover {
    /// A closed popover whose panel is named `name` (what a screen reader says: "Filters").
    pub fn new(labels: &Labels, name: &str) -> Self {
        Self {
            id: "popover".to_owned(),
            name: name.to_owned(),
            labels: labels.clone(),
            open: Arc::new(AtomicBool::new(false)),
            live: Live::new(),
            focus_to: None,
            on_close: None,
        }
    }

    /// The prefix of the ids of the parts: `{id}-trigger`, `{id}-panel`, `{id}-close`.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// How the host gives the keyboard to an element: asked for the trigger when the popover closes.
    #[must_use]
    pub fn focus_to(mut self, host: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.focus_to = Some(Arc::new(host));
        self
    }

    /// Call `handler` after the popover closed.
    #[must_use]
    pub fn on_close(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }

    /// The id of the trigger.
    pub fn trigger_id(&self) -> String {
        format!("{}-trigger", self.id)
    }

    /// The id of the panel, as the trigger names it in `aria-controls`.
    pub fn panel_id(&self) -> String {
        format!("{}-panel", self.id)
    }

    /// Whether the panel is open.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    /// Open the panel.
    pub fn open(&self) {
        self.open.store(true, Ordering::Relaxed);
        self.live.refresh_now();
    }

    /// Close the panel and give the keyboard back to the trigger; false (nothing happens) when it was closed.
    pub fn close(&self) -> bool {
        if !self.open.swap(false, Ordering::Relaxed) {
            return false;
        }
        self.live.refresh_now();
        if let Some(host) = &self.focus_to {
            host(&self.trigger_id());
        }
        if let Some(handler) = &self.on_close {
            handler();
        }
        true
    }

    /// Open when closed, close when open.
    pub fn toggle(&self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }

    /// The trigger and, while open, the panel under it. `trigger` is a tab stop (a button); it gets the id, the ARIA state and the
    /// click. `content` is what the panel holds.
    pub fn build(
        &self,
        trigger: impl Fn() -> Div + Send + Sync + 'static,
        content: impl Fn() -> Div + Send + Sync + 'static,
    ) -> Div {
        let this = self.clone();
        self.live
            .view("popover", move || this.render(trigger(), content()))
    }

    fn render(&self, trigger: Div, content: Div) -> Div {
        let open = self.is_open();
        let (click, key) = (self.clone(), self.clone());
        let trigger = trigger
            .id(self.trigger_id())
            .attr("aria-haspopup", "dialog")
            .attr("aria-expanded", if open { "true" } else { "false" })
            .attr("aria-controls", self.panel_id())
            .on_click(move |_| click.toggle());
        let mut host = div()
            .id(format!("{}-host", self.id))
            .class("fs-popover-host");
        if open {
            host = host.class("fs-popover-open");
        }
        host = host
            .on_key_down(move |ctx| {
                if ctx.key_code == 27 {
                    key.close();
                }
            })
            .child(trigger);
        if open {
            let closer = self.clone();
            let mark = bind_keys(
                div()
                    .id(format!("{}-close", self.id))
                    .class("fs-popover-close")
                    .attr("role", "button")
                    .attr("aria-label", &self.labels.close)
                    .child(icon_stroked(Glyph::Close, 14.0, Tone::Normal, Some(STROKE))),
                Arc::new(move |event| match event.key {
                    Key::Escape | Key::Activate | Key::Enter | Key::Space => closer.close(),
                    _ => false,
                }),
            );
            let click_close = self.clone();
            host = host.child(
                div()
                    .id(self.panel_id())
                    .class("fs-popover")
                    .attr("role", "dialog")
                    .attr("aria-label", &self.name)
                    .child(
                        div()
                            .class("fs-popover-head")
                            .child(mark.on_click(move |_| {
                                click_close.close();
                            })),
                    )
                    .child(content),
            );
        }
        host
    }
}
