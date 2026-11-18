//! `alertdialog`: a dialog that interrupts with a message and wants an answer (WAI-ARIA APG, alertdialog pattern).
//!
//! Pattern: Command + Template method over the shared frame (`frame.rs`). Role `alertdialog` (not `dialog`) with `aria-modal`,
//! `aria-labelledby` (the title) and `aria-describedby` (the message). With one button ("OK") Enter and Escape both close it;
//! with a cancel button too the keyboard starts on the LEAST destructive one (cancel) and Enter alone does nothing. A widget cannot
//! move the keyboard, so [`AlertDialog::focus_first`] asks the host through [`FocusTo`] (see `focus.rs`).

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::frame::{Act, Parts, button, frame};
use super::labels::Labels;
use crate::Element;
use crate::focus::FocusTo;

/// The dialog. Clones share the callbacks.
#[derive(Clone)]
pub struct AlertDialog {
    id: String,
    title: String,
    message: String,
    ok: String,
    cancel: Option<String>,
    danger: bool,
    labels: Labels,
    on_ok: Option<Act>,
    on_cancel: Option<Act>,
    focus_to: Option<FocusTo>,
}

impl AlertDialog {
    /// A dialog with a `title` and the `message` that interrupts; one button "OK" from `labels`.
    pub fn new(labels: &Labels, title: &str, message: &str) -> Self {
        Self {
            id: "alert-dialog".to_owned(),
            title: title.to_owned(),
            message: message.to_owned(),
            ok: labels.ok.clone(),
            cancel: None,
            danger: false,
            labels: labels.clone(),
            on_ok: None,
            on_cancel: None,
            focus_to: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The word on the answering button ("Delete").
    #[must_use]
    pub fn ok_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.ok);
        self
    }

    /// Add a cancel button (the standard word, or `word` when given); the keyboard then starts on it.
    #[must_use]
    pub fn with_cancel(mut self, word: Option<&str>) -> Self {
        self.cancel = Some(word.map_or_else(|| self.labels.cancel.clone(), str::to_owned));
        self
    }

    /// Mark the answer as destructive.
    #[must_use]
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Call `handler` on the answering button (and on Enter or Escape when there is no cancel button).
    #[must_use]
    pub fn on_ok(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_ok = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on the cancel button or Escape (with a cancel button).
    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    /// How the host gives the keyboard to an element.
    #[must_use]
    pub fn focus_to(mut self, host: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.focus_to = Some(Arc::new(host));
        self
    }

    /// The id of the element that has the keyboard first: the cancel button when there is one (the least destructive), else "OK".
    pub fn first_focus_id(&self) -> String {
        let tag = if self.cancel.is_some() {
            "cancel"
        } else {
            "ok"
        };
        format!("{}-{tag}", self.id)
    }

    /// Ask the host to give the keyboard to [`AlertDialog::first_focus_id`]; call it when the dialog opens.
    pub fn focus_first(&self) {
        if let Some(host) = &self.focus_to {
            host(&self.first_focus_id());
        }
    }

    /// Answer with the main button.
    pub fn answer(&self) {
        if let Some(handler) = &self.on_ok {
            handler();
        }
    }

    /// Leave: cancel with a cancel button, else the same as the answer.
    pub fn leave(&self) {
        let handler = if self.cancel.is_some() {
            &self.on_cancel
        } else {
            &self.on_ok
        };
        if let Some(handler) = handler {
            handler();
        }
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        self.render()
    }

    fn render(&self) -> Div {
        let (yes, no, key_yes, key_no) = (self.clone(), self.clone(), self.clone(), self.clone());
        let variant = if self.danger {
            ButtonVariant::Destructive
        } else {
            ButtonVariant::Primary
        };
        let mut footer = Vec::new();
        if let Some(word) = &self.cancel {
            footer.push(button(
                &self.id,
                "cancel",
                word,
                ButtonVariant::Outline,
                false,
                Arc::new(move || no.leave()),
            ));
        }
        footer.push(button(
            &self.id,
            "ok",
            &self.ok,
            variant,
            false,
            Arc::new(move || yes.answer()),
        ));
        let enter: Option<Act> = if self.cancel.is_some() {
            None
        } else {
            Some(Arc::new(move || key_yes.answer()))
        };
        frame(Parts {
            id: self.id.clone(),
            kind: "alert",
            role: "alertdialog",
            states: if self.danger {
                vec!["danger"]
            } else {
                Vec::new()
            },
            title: self.title.clone(),
            body: div().child(text(self.message.clone()).class("fs-dlg-text")),
            footer,
            enter,
            escape: Some(Arc::new(move || key_no.leave())),
        })
    }
}
