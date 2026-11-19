//! One disclosure.
//!
//! Pattern: Facade + Observer — a clonable handle over the open mark; a change draws the disclosure again through `Live`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::{Div, div};

use super::parts::{Press, header, panel};
use crate::live::Live;

type OnToggle = Arc<dyn Fn(bool) + Send + Sync>;

/// A disclosure. Clones share the mark and the callback.
#[derive(Clone)]
pub struct Disclosure {
    id: String,
    title: String,
    open: Arc<AtomicBool>,
    live: Live,
    on_toggle: Option<OnToggle>,
}

impl Disclosure {
    /// A closed disclosure with this title; ids: `<id>` (header) and `<id>-panel`.
    pub fn new(title: &str) -> Self {
        Self {
            id: "disclosure".to_owned(),
            title: title.to_owned(),
            open: Arc::new(AtomicBool::new(false)),
            live: Live::new(),
            on_toggle: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Start open.
    #[must_use]
    pub fn opened(self) -> Self {
        self.open.store(true, Ordering::Relaxed);
        self
    }

    /// Call `handler` with the new state after a press.
    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    /// Whether the content is shown.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    /// Show or hide the content from the program.
    pub fn set_open(&self, open: bool) {
        if self.open.swap(open, Ordering::Relaxed) != open {
            self.live.refresh_now();
        }
    }

    /// What a press does: turn the state and tell the caller.
    pub fn toggle(&self) {
        let now = !self.open.fetch_xor(true, Ordering::Relaxed);
        self.live.refresh_now();
        if let Some(handler) = &self.on_toggle {
            handler(now);
        }
    }

    /// The disclosure; `content` is built when it is shown.
    pub fn build(&self, content: impl Fn() -> Div + Send + Sync + 'static) -> Div {
        let this = self.clone();
        self.live.view("disclosure", move || this.render(&content))
    }

    fn render(&self, content: &(dyn Fn() -> Div + Send + Sync)) -> Div {
        let open = self.is_open();
        let panel_id = format!("{}-panel", self.id);
        let press: Press = {
            let this = self.clone();
            Arc::new(move || this.toggle())
        };
        let mut host = div()
            .id(format!("{}-host", self.id))
            .class("fs-disclosure")
            .child(header(
                &self.id,
                &panel_id,
                &self.title,
                open,
                false,
                &press,
            ));
        if open {
            host = host.class("fs-disclosure-open").child(panel(
                &panel_id,
                &self.id,
                false,
                content(),
            ));
        }
        host
    }
}
