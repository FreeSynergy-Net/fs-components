//! `empty-state`: what a list or a page shows when there is nothing: a glyph, a title, a line of text, optionally an action.
//!
//! Pattern: Builder + Adapter. The words are the caller's (they say what is missing and what to do); the surface knows none.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Element;
use crate::icon::{Glyph, Tone, icon};

/// The empty surface.
#[derive(Clone)]
pub struct EmptyState {
    id: String,
    glyph: Option<Glyph>,
    title: String,
    text: Option<String>,
    action: Option<(String, Arc<dyn Fn() + Send + Sync>)>,
}

impl EmptyState {
    /// A surface with a `title` ("No appointments in this period").
    pub fn new(title: &str) -> Self {
        Self {
            id: "empty-state".to_owned(),
            glyph: None,
            title: title.to_owned(),
            text: None,
            action: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A glyph above the title.
    #[must_use]
    pub fn glyph(mut self, glyph: Glyph) -> Self {
        self.glyph = Some(glyph);
        self
    }

    /// A line below the title.
    #[must_use]
    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_owned());
        self
    }

    /// A button with `label` ("Create appointment", "Clear search") that runs `handler`.
    #[must_use]
    pub fn action(mut self, label: &str, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.action = Some((label.to_owned(), Arc::new(handler)));
        self
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        self.render()
    }

    pub(super) fn render(&self) -> Div {
        let mut root = div().id(&self.id).class("fs-empty");
        if let Some(glyph) = self.glyph {
            root = root.child(
                div()
                    .class("fs-empty-icon")
                    .child(icon(glyph, 32.0, Tone::Muted)),
            );
        }
        root = root.child(
            text(self.title.clone())
                .id(format!("{}-title", self.id))
                .class("fs-empty-title"),
        );
        if let Some(line) = &self.text {
            root = root.child(
                text(line.clone())
                    .id(format!("{}-text", self.id))
                    .class("fs-empty-text"),
            );
        }
        if let Some((label, act)) = &self.action {
            let act = Arc::clone(act);
            let key = format!("{}-action", self.id);
            root = root.child(
                ButtonBuilder::with_key(key.clone(), label.clone())
                    .variant(ButtonVariant::Primary)
                    .on_click(move |_| act())
                    .build_component()
                    .id(&key),
            );
        }
        root
    }
}
