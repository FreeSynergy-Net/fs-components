//! `license-dialog`: the licence text word for word, scrollable, with Accept and Decline (and optionally a button that hands the
//! text to the clipboard).
//!
//! Pattern: Command (`accept`, `decline`, `copy`) + Adapter over Blinc's scroll container (`ScrollRef` for the keys). The text is
//! shown as given: no line is cut, joined or translated (it is a legal text, ADR 0030: what was received stays the original).
//! The text area is a tab stop; Up, Down, `PageUp`, `PageDown`, Home and End scroll it. Enter alone does NOT accept (an agreement
//! needs a deliberate press on the button); Escape declines.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::Scroll;

use super::frame::{Act, Parts, button, frame};
use super::labels::Labels;
use crate::Element;
use crate::keys::{Key, bind_keys};

/// The height of the text area in pixels (the dialog grows no further, the text scrolls).
pub const TEXT_HEIGHT: f32 = 260.0;
/// One step of an arrow key, in pixels.
const LINE_STEP: f32 = 40.0;

/// Called with the whole text.
type OnCopy = Arc<dyn Fn(&str) + Send + Sync>;

/// The dialog. Clones share the callbacks and the scroll place.
#[derive(Clone)]
pub struct LicenseDialog {
    id: String,
    title: String,
    licence: String,
    text: Arc<String>,
    accept: String,
    decline: String,
    copy: String,
    scroll: ScrollRef,
    on_accept: Option<Act>,
    on_decline: Option<Act>,
    on_copy: Option<OnCopy>,
}

impl LicenseDialog {
    /// A dialog for the licence `licence` (its name, "SIL OFL 1.1") with its full `text`.
    pub fn new(labels: &Labels, title: &str, licence: &str, text: &str) -> Self {
        Self {
            id: "license-dialog".to_owned(),
            title: title.to_owned(),
            licence: licence.to_owned(),
            text: Arc::new(text.to_owned()),
            accept: labels.accept.clone(),
            decline: labels.decline.clone(),
            copy: labels.copy.clone(),
            scroll: fs_components_primitives::scroll_ref(),
            on_accept: None,
            on_decline: None,
            on_copy: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Call `handler` on the Accept button.
    #[must_use]
    pub fn on_accept(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_accept = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on the Decline button or Escape.
    #[must_use]
    pub fn on_decline(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_decline = Some(Arc::new(handler));
        self
    }

    /// Show the copy button; `handler` gets the whole text (the frontend puts it on the clipboard).
    #[must_use]
    pub fn on_copy(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_copy = Some(Arc::new(handler));
        self
    }

    /// The text, as given.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Agree.
    pub fn accept(&self) {
        if let Some(handler) = &self.on_accept {
            handler();
        }
    }

    /// Do not agree.
    pub fn decline(&self) {
        if let Some(handler) = &self.on_decline {
            handler();
        }
    }

    /// Hand the text to the copy callback; false without one.
    pub fn copy(&self) -> bool {
        let Some(handler) = &self.on_copy else {
            return false;
        };
        handler(&self.text);
        true
    }

    /// How far the text is scrolled, in pixels from the top.
    pub fn scrolled(&self) -> f32 {
        self.scroll.scroll_y()
    }

    /// `ScrollRef::scroll_by` takes the delta of the wheel: a negative `dy` moves the text up, towards its end.
    fn keys(&self, key: Key) -> bool {
        let page = self.scroll.viewport_size().map_or(TEXT_HEIGHT, |v| v.1) - LINE_STEP;
        match key {
            Key::Up => self.scroll.scroll_by(0.0, LINE_STEP),
            Key::Down => self.scroll.scroll_by(0.0, -LINE_STEP),
            Key::PageUp => self.scroll.scroll_by(0.0, page),
            Key::PageDown => self.scroll.scroll_by(0.0, -page),
            Key::Home => self.scroll.scroll_to_top(),
            Key::End => self.scroll.scroll_to_bottom(),
            _ => return false,
        }
        true
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        let mut lines = div().class("fs-lic-lines");
        for line in self.text.split('\n') {
            // an empty line is kept as a line: the paragraphs of the text stay where they are
            let shown = if line.is_empty() { " " } else { line };
            lines = lines.child(text(shown.to_owned()).class("fs-lic-line"));
        }
        let area = Scroll::new()
            .bind(&self.scroll)
            .w_full()
            .h(TEXT_HEIGHT)
            .content(lines);
        let this = self.clone();
        let area = bind_keys(
            div()
                .id(format!("{}-text", self.id))
                .class("fs-lic-text")
                .attr("role", "document")
                .child(area),
            Arc::new(move |event| this.keys(event.key)),
        );
        let body = div()
            .flex_col()
            .gap(1.0)
            .child(text(self.licence.clone()).class("fs-lic-name"))
            .child(area);
        let (yes, no, cp, esc) = (self.clone(), self.clone(), self.clone(), self.clone());
        let mut footer = Vec::new();
        footer.push(button(
            &self.id,
            "decline",
            &self.decline,
            ButtonVariant::Outline,
            false,
            Arc::new(move || no.decline()),
        ));
        if self.on_copy.is_some() {
            footer.push(button(
                &self.id,
                "copy",
                &self.copy,
                ButtonVariant::Ghost,
                false,
                Arc::new(move || {
                    cp.copy();
                }),
            ));
        }
        footer.push(button(
            &self.id,
            "accept",
            &self.accept,
            ButtonVariant::Primary,
            false,
            Arc::new(move || yes.accept()),
        ));
        frame(Parts {
            id: self.id.clone(),
            kind: "license",
            role: "dialog",
            describe: false,
            states: Vec::new(),
            title: self.title.clone(),
            body,
            footer,
            enter: None,
            escape: Some(Arc::new(move || esc.decline())),
        })
    }
}
