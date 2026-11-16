//! `alert`: a message the person must see, with an optional action. The failed surface is an alert of the step `Error` with the
//! action "Try again".
//!
//! Pattern: Builder + Adapter over the `blinc_cn` button. Role `alert` for warnings and errors, `status` for the rest
//! (`Severity::role`). The step is told by glyph and by a heading word, not by colour alone.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::STROKE;
use super::labels::Labels;
use super::severity::Severity;
use crate::Element;
use crate::icon::{Tone, icon_stroked};

/// What the action button does.
pub type Act = Arc<dyn Fn() + Send + Sync>;

/// A message with a glyph, an optional title and an optional action.
#[derive(Clone)]
pub struct Alert {
    id: String,
    severity: Severity,
    title: Option<String>,
    text: String,
    action: Option<(String, Act)>,
}

impl Alert {
    /// An alert of `severity` with `text`.
    pub fn new(severity: Severity, text: &str) -> Self {
        Self {
            id: "alert".to_owned(),
            severity,
            title: None,
            text: text.to_owned(),
            action: None,
        }
    }

    /// The failed surface: title "Could not load", `reason` below, and "Try again" calling `retry`.
    pub fn failed(labels: &Labels, reason: &str, retry: impl Fn() + Send + Sync + 'static) -> Self {
        Self::new(Severity::Error, reason)
            .title(&labels.failed)
            .action(&labels.retry, retry)
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A line above the text.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// A button with `label` that runs `handler`.
    #[must_use]
    pub fn action(mut self, label: &str, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.action = Some((label.to_owned(), Arc::new(handler)));
        self
    }

    /// The step of the alert.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        self.render()
    }

    pub(super) fn render(&self) -> Div {
        let mut words = div().class("fs-alert-words");
        if let Some(title) = &self.title {
            words = words.child(
                text(title.clone())
                    .id(format!("{}-title", self.id))
                    .class("fs-alert-title"),
            );
        }
        words = words.child(
            text(self.text.clone())
                .id(format!("{}-text", self.id))
                .class("fs-alert-text"),
        );
        let mut root = div()
            .id(&self.id)
            .class("fs-alert")
            .class(format!("fs-alert-{}", self.severity.name()))
            .attr("role", self.severity.role())
            .child(div().class("fs-alert-icon").child(icon_stroked(
                self.severity.glyph(),
                18.0,
                Tone::Normal,
                Some(STROKE),
            )))
            .child(words);
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
        root
    }
}
