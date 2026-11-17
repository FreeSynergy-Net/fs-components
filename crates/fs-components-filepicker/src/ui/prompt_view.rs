//! The view of the path field: the fallback of the file dialog, marked as provisional.
//!
//! Pattern: Observer (the prompt tells the view to draw again) + Adapter over the search field of the primitives. The field's
//! text lives in the field's own state, outside what is drawn again, so a refused path does not lose what was typed. Nothing
//! is shown while no question is open. The marker "provisional" is always there; a refused path shows its problem.

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_widgets::Element;
use fs_components_widgets::live::Live;

use crate::labels::Labels;
use crate::prompt::PathPrompt;

/// The view. Clones share the field and the prompt.
#[derive(Clone)]
pub struct PathPromptView {
    id: String,
    labels: Labels,
    prompt: PathPrompt,
    field: SearchField,
    live: Live,
}

impl PathPromptView {
    /// A view over `prompt`; Enter in the field and the OK button take the path, Cancel (or Escape) leaves.
    pub fn new(labels: &Labels, prompt: &PathPrompt) -> Self {
        let live = Live::new();
        prompt.on_change(move || live.refresh_now());
        let taker = prompt.clone();
        let field = SearchField::new(&labels.path_placeholder).size(FieldSize::Body);
        let typed = field.clone();
        let field = field.on_submit(std::sync::Arc::new(move || {
            let _ = taker.submit(&typed.text());
        }));
        Self {
            id: "file-path-prompt".to_owned(),
            labels: labels.clone(),
            prompt: prompt.clone(),
            field,
            live,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.field.text()
    }

    /// Take what is typed (the OK button and Enter call this).
    pub fn confirm(&self) -> bool {
        self.prompt.submit(&self.field.text()).is_ok()
    }

    /// Leave without a path.
    pub fn cancel(&self) {
        self.prompt.cancel();
    }

    /// The element to place in a layout or a modal.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.region("file-path-prompt", move || this.render())
    }

    fn render(&self) -> blinc_layout::div::Div {
        let Some(request) = self.prompt.pending() else {
            return div().id(&self.id);
        };
        let (ok, no, keys) = (self.clone(), self.clone(), self.clone());
        let id = &self.id;
        let button = |tag: &str, word: &str, variant| {
            ButtonBuilder::with_key(format!("{id}-{tag}"), word.to_owned()).variant(variant)
        };
        let mut root = div()
            .id(id)
            .class("fs-fp-prompt")
            .child(text(request.title).class("fs-fp-title"))
            .child(
                div()
                    .id(format!("{id}-provisional"))
                    .child(text(self.labels.provisional.clone()).class("fs-fp-provisional")),
            )
            .child(div().id(format!("{id}-field")).child(self.field.build()));
        if let Some(problem) = self.prompt.problem() {
            root = root.child(
                div()
                    .id(format!("{id}-problem"))
                    .child(text(self.labels.problem(&problem)).class("fs-fp-problem")),
            );
        }
        root.child(
            div()
                .class("fs-fp-buttons")
                .child(
                    button("cancel", &self.labels.cancel, ButtonVariant::Outline)
                        .on_click(move |_| no.cancel())
                        .build_component()
                        .id(&format!("{id}-cancel")),
                )
                .child(
                    button("ok", &self.labels.ok, ButtonVariant::Primary)
                        .on_click(move |_| {
                            ok.confirm();
                        })
                        .build_component()
                        .id(&format!("{id}-ok")),
                ),
        )
        .on_key_down(move |ctx| match ctx.key_code {
            13 => {
                keys.confirm();
            }
            27 => keys.cancel(),
            _ => {}
        })
    }
}
