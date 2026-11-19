//! The help as an element.
//!
//! Pattern: Builder + Adapter over the `blinc_cn` button. `Help::build(topic)` asks the source and draws the outcome; whatever comes
//! back, the element has words in it (ADR 0097: an entry with content or a visible fallback, never greyed out, never empty).

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::model::{HelpBlock, HelpSource, Outcome, paragraphs, resolve};
use crate::Element;

/// What the documentation button does.
type Act = Arc<dyn Fn() + Send + Sync>;

/// The help of one program in one window.
#[derive(Clone)]
pub struct Help {
    id: String,
    program: String,
    labels: Labels,
    source: Arc<dyn HelpSource>,
    langs: Vec<String>,
    on_docs: Option<Act>,
}

impl Help {
    /// The help of `program`, read from `source`, in the languages of `labels` (the wish list with its fallback: `de-AT`, `de`, `en`).
    pub fn new(labels: &Labels, program: &str, source: Arc<dyn HelpSource>) -> Self {
        // `Locale::chain` does not end in English: the caller does (ADR 0057 chain: language, shorter forms, English)
        let mut langs: Vec<String> = labels
            .locale
            .chain()
            .iter()
            .map(|l| l.as_str().to_owned())
            .collect();
        if !langs.iter().any(|l| l == "en") {
            langs.push("en".to_owned());
        }
        Self {
            id: "help".to_owned(),
            program: program.to_owned(),
            langs,
            labels: labels.clone(),
            source,
            on_docs: None,
        }
    }

    /// The prefix of the ids of the parts: `{id}` the root, `{id}-text`, `{id}-note`, `{id}-docs`.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The languages wished, first the best (a setting; the default is the chain of the language of the labels).
    #[must_use]
    pub fn languages(mut self, langs: &[&str]) -> Self {
        self.langs = langs.iter().map(|l| (*l).to_owned()).collect();
        self
    }

    /// The way out when there is no entry: the button "Open the documentation" runs `handler` (the host opens the page on the
    /// documentation site, ADR 0065 point 3).
    #[must_use]
    pub fn on_docs(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_docs = Some(Arc::new(handler));
        self
    }

    /// What the source says about `topic`.
    pub fn outcome(&self, topic: &str) -> Outcome {
        resolve(self.source.as_ref(), &self.program, topic, &self.langs)
    }

    /// The first sentence the person reads for `topic`: the first paragraph of the entry, or the sentence of the fallback. Never
    /// empty.
    pub fn visible_text(&self, topic: &str) -> String {
        match self.outcome(topic) {
            Outcome::Found(block) => paragraphs(&block.text)
                .into_iter()
                .next()
                .unwrap_or_else(|| self.labels.missing(topic)),
            Outcome::Missing => self.labels.missing(topic),
            Outcome::Unavailable(_) => self.labels.unavailable(),
        }
    }

    /// The element for `topic`.
    pub fn build(&self, topic: &str) -> impl Element + use<> {
        self.render(&self.outcome(topic), topic)
    }

    fn render(&self, outcome: &Outcome, topic: &str) -> Div {
        let kind = match outcome {
            Outcome::Found(_) => "found",
            Outcome::Missing => "missing",
            Outcome::Unavailable(_) => "unavailable",
        };
        let mut root = div()
            .id(&self.id)
            .class("fs-help")
            .class(format!("fs-help-{kind}"));
        match outcome {
            Outcome::Found(block) => root = self.entry(root, block),
            Outcome::Missing => root = self.fallback(root, &self.labels.missing(topic)),
            Outcome::Unavailable(_) => root = self.fallback(root, &self.labels.unavailable()),
        }
        root
    }

    fn entry(&self, mut root: Div, block: &HelpBlock) -> Div {
        for (i, paragraph) in paragraphs(&block.text).into_iter().enumerate() {
            let mut line = text(paragraph).class("fs-help-text");
            if i == 0 {
                line = line.id(format!("{}-text", self.id));
            }
            root = root.child(line);
        }
        if block.fallback_from.is_some() {
            root = root.child(
                text(self.labels.shown_in(&block.lang))
                    .id(format!("{}-note", self.id))
                    .class("fs-help-note"),
            );
        }
        if block.outdated {
            root = root.child(
                text(self.labels.outdated())
                    .id(format!("{}-outdated", self.id))
                    .class("fs-help-note"),
            );
        }
        root
    }

    fn fallback(&self, mut root: Div, sentence: &str) -> Div {
        root = root.child(
            text(sentence.to_owned())
                .id(format!("{}-text", self.id))
                .class("fs-help-text"),
        );
        if let Some(act) = &self.on_docs {
            let act = Arc::clone(act);
            let key = format!("{}-docs", self.id);
            root = root.child(
                div().class("fs-help-action").child(
                    ButtonBuilder::with_key(key.clone(), self.labels.open_docs.clone())
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Small)
                        .on_click(move |_| act())
                        .build_component()
                        .id(&key),
                ),
            );
        }
        root
    }
}
