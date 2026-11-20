//! `input-dialog`: asks for one line of text (the name of a column, the title of a card) and hands it over trimmed.
//!
//! Pattern: Command (`confirm`, `cancel` are what the buttons and the keys call) + Adapter over the search field of the primitives
//! (its input already edits). The field keeps its text outside anything that is drawn, so typing never loses the keyboard. The dialog
//! knows no data: a blank line is no answer (`confirm` is false, the dialog stays), anything else goes to the caller.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_primitives::{FieldSize, SearchField};

use super::frame::{Act, Parts, button, frame};
use super::labels::Labels;
use crate::Element;

/// Called with the trimmed line when the person confirms.
pub type OnInput = Arc<dyn Fn(&str) + Send + Sync>;

/// The dialog. Clones share the field and the callbacks. The frontend puts [`InputDialog::build`] in its modal
/// (or in [`super::modal`]).
#[derive(Clone)]
pub struct InputDialog {
    id: String,
    title: String,
    message: Option<String>,
    ok: String,
    cancel: String,
    field: SearchField,
    on_input: Option<OnInput>,
    on_cancel: Option<Act>,
}

impl InputDialog {
    /// A dialog with a `title` ("New column") and a `placeholder` shown while the field is empty; buttons from `labels`.
    pub fn new(labels: &Labels, title: &str, placeholder: &str) -> Self {
        Self {
            id: "input-dialog".to_owned(),
            title: title.to_owned(),
            message: None,
            ok: labels.ok.clone(),
            cancel: labels.cancel.clone(),
            field: SearchField::new(placeholder).size(FieldSize::Body),
            on_input: None,
            on_cancel: None,
        }
    }

    /// The prefix of the ids of the parts (several dialogs in one window).
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A line of text above the field (what the line is for).
    #[must_use]
    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_owned());
        self
    }

    /// The word on the confirm button ("Add").
    #[must_use]
    pub fn confirm_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.ok);
        self
    }

    /// Start with `value` in the field, all of it selected (a rename).
    #[must_use]
    pub fn initial(self, value: &str) -> Self {
        self.field.set_text(value);
        self
    }

    /// Call `handler` with the trimmed line on the confirm button or Enter.
    #[must_use]
    pub fn on_input(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_input = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on the cancel button or Escape.
    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.field.text()
    }

    /// Ask for the keyboard in the field (call when the dialog is shown).
    pub fn focus(&self) {
        self.field.focus();
    }

    /// Take the line: false (nothing happens, the dialog stays) while the field holds only blanks.
    pub fn confirm(&self) -> bool {
        let line = self.field.text();
        let line = line.trim();
        if line.is_empty() {
            return false;
        }
        if let Some(handler) = &self.on_input {
            handler(line);
        }
        true
    }

    /// Leave without a line.
    pub fn cancel(&self) {
        if let Some(handler) = &self.on_cancel {
            handler();
        }
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        let (yes, no, enter) = (self.clone(), self.clone(), self.clone());
        let cancel = self.clone();
        let mut body = div().class("fs-dlg-input-body");
        if let Some(message) = &self.message {
            body = body.child(text(message.clone()).class("fs-dlg-input-text"));
        }
        body = body.child(
            div()
                .id(format!("{}-field", self.id))
                .class("fs-dlg-input-field")
                .child(self.field.build()),
        );
        frame(Parts {
            id: self.id.clone(),
            kind: "input",
            role: "dialog",
            describe: false,
            states: Vec::new(),
            title: self.title.clone(),
            body,
            footer: vec![
                button(
                    &self.id,
                    "cancel",
                    &self.cancel,
                    ButtonVariant::Outline,
                    false,
                    Arc::new(move || cancel.cancel()),
                ),
                button(
                    &self.id,
                    "ok",
                    &self.ok,
                    ButtonVariant::Primary,
                    false,
                    Arc::new(move || {
                        yes.confirm();
                    }),
                ),
            ],
            // The field keeps the keyboard while typing; Enter and Escape reach the frame as broadcast keys.
            enter: Some(Arc::new(move || {
                enter.confirm();
            })),
            escape: Some(Arc::new(move || no.cancel())),
        })
    }
}
