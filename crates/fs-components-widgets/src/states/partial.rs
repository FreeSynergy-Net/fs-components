//! `partial-notice`: the note above a result that is not whole: "2 of 9 could not be shown", the names of what is missing and
//! an action to try those again.
//!
//! Pattern: Builder + Adapter. A warning, never an error: what is there is shown and usable. Every missing name is listed (the
//! caller passes the few that matter; a long list is its choice).

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::STROKE;
use super::labels::Labels;
use super::severity::Severity;
use crate::Element;
use crate::icon::{Tone, icon_stroked};

/// The notice.
#[derive(Clone)]
pub struct PartialNotice {
    id: String,
    lead: String,
    missing: Vec<String>,
    retry: Option<(String, Arc<dyn Fn() + Send + Sync>)>,
}

impl PartialNotice {
    /// A notice that `missing.len()` of `total` could not be shown; `missing` are their names (a calendar, a source).
    pub fn new(labels: &Labels, missing: &[String], total: usize) -> Self {
        Self {
            id: "partial-notice".to_owned(),
            lead: labels.partial(missing.len(), total),
            missing: missing.to_vec(),
            retry: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The "Try again" button of `labels`, running `handler`.
    #[must_use]
    pub fn retry(mut self, labels: &Labels, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.retry = Some((labels.retry.clone(), Arc::new(handler)));
        self
    }

    /// The sentence.
    pub fn lead(&self) -> &str {
        &self.lead
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        self.render()
    }

    pub(super) fn render(&self) -> Div {
        let mut list = div().class("fs-partial-list");
        for name in &self.missing {
            list = list.child(text(name.clone()).class("fs-partial-item"));
        }
        let words = div()
            .class("fs-partial-words")
            .child(
                text(self.lead.clone())
                    .id(format!("{}-text", self.id))
                    .class("fs-partial-text"),
            )
            .child(list);
        let mut root = div()
            .id(&self.id)
            .class("fs-partial")
            .attr("role", Severity::Warn.role())
            .child(div().class("fs-partial-icon").child(icon_stroked(
                Severity::Warn.glyph(),
                18.0,
                Tone::Normal,
                Some(STROKE),
            )))
            .child(words);
        if let Some((label, act)) = &self.retry {
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
