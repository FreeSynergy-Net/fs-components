//! The banner as an element and its handle.
//!
//! Pattern: Builder + Observer — a clonable handle over the open mark; dismissing draws it again (as nothing) through `Live`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::labels::Labels;
use crate::Element;
use crate::icon::{Glyph, Tone, icon_stroked};
use crate::keys::{Key, bind_keys};
use crate::live::Live;
use crate::states::{STROKE, Severity};

/// What a button or the dismissal does.
type Act = Arc<dyn Fn() + Send + Sync>;

/// A banner. Clones share the open mark and the callbacks.
#[derive(Clone)]
pub struct Banner {
    id: String,
    severity: Severity,
    words: String,
    labels: Labels,
    action: Option<(String, Act)>,
    dismissible: bool,
    on_dismiss: Option<Act>,
    open: Arc<AtomicBool>,
    live: Live,
}

impl Banner {
    /// An open banner of `severity` saying `words`; it can be dismissed.
    pub fn new(labels: &Labels, severity: Severity, words: &str) -> Self {
        Self {
            id: "banner".to_owned(),
            severity,
            words: words.to_owned(),
            labels: labels.clone(),
            action: None,
            dismissible: true,
            on_dismiss: None,
            open: Arc::new(AtomicBool::new(true)),
            live: Live::new(),
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A button with `label` that runs `handler`.
    #[must_use]
    pub fn action(mut self, label: &str, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.action = Some((label.to_owned(), Arc::new(handler)));
        self
    }

    /// Whether the banner has a dismiss mark (default yes; a banner about something that must be dealt with has none).
    #[must_use]
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Call `handler` after the banner was dismissed.
    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(handler));
        self
    }

    /// The step of the banner.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Whether the banner shows.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    /// Show the banner again.
    pub fn show(&self) {
        self.open.store(true, Ordering::Relaxed);
        self.live.refresh_now();
    }

    /// Hide the banner; false (nothing happens) when it was hidden or cannot be dismissed.
    pub fn dismiss(&self) -> bool {
        if !self.dismissible || !self.open.swap(false, Ordering::Relaxed) {
            return false;
        }
        self.live.refresh_now();
        if let Some(handler) = &self.on_dismiss {
            handler();
        }
        true
    }

    /// Hide the banner whoever may dismiss it (the program deals with the cause itself).
    pub fn resolve(&self) {
        if self.open.swap(false, Ordering::Relaxed) {
            self.live.refresh_now();
        }
    }

    /// The element: the banner, or nothing while it is hidden.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.view("banner", move || this.render())
    }

    fn render(&self) -> Div {
        if !self.is_open() {
            return div();
        }
        let mut root = div()
            .id(&self.id)
            .class("fs-banner")
            .class(format!("fs-banner-{}", self.severity.name()))
            .attr("role", self.severity.role())
            .child(div().class("fs-banner-icon").child(icon_stroked(
                self.severity.glyph(),
                18.0,
                Tone::Normal,
                Some(STROKE),
            )))
            .child(
                text(self.words.clone())
                    .id(format!("{}-text", self.id))
                    .class("fs-banner-text"),
            );
        if let Some((label, act)) = &self.action {
            let act = Arc::clone(act);
            let key = format!("{}-action", self.id);
            root = root.child(
                ButtonBuilder::with_key(key.clone(), label.clone())
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Small)
                    .on_click(move |_| act())
                    .build_component()
                    .id(&key),
            );
        }
        if self.dismissible {
            let (keys, click) = (self.clone(), self.clone());
            let mark = bind_keys(
                div()
                    .id(format!("{}-close", self.id))
                    .class("fs-banner-close")
                    .attr("role", "button")
                    .attr("aria-label", &self.labels.close)
                    .child(icon_stroked(Glyph::Close, 14.0, Tone::Normal, Some(STROKE))),
                Arc::new(move |event| match event.key {
                    Key::Escape | Key::Activate | Key::Enter => keys.dismiss(),
                    _ => false,
                }),
            )
            .on_click(move |_| {
                click.dismiss();
            });
            root = root.child(mark);
        }
        root
    }
}
