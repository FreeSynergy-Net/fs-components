//! `confirm-dialog`: asks once before something that cannot be taken back, or that the person may not want.
//!
//! Pattern: Command (`confirm`, `cancel` are what the buttons and the keys call) + Observer (`Live` draws it again when it
//! becomes busy). States: default, `danger` (the confirm button is the destructive one and Enter alone does nothing: the
//! person must reach the button), `busy` (the answer is being carried out: both buttons off, Escape ignored).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::div;
use blinc_layout::prelude::text;

use super::frame::{Act, Parts, button, frame};
use super::labels::Labels;
use crate::Element;
use crate::live::Live;

/// The dialog. Clones share the busy mark and the callbacks. The frontend puts [`ConfirmDialog::build`] in its modal
/// (or in [`super::modal`]).
#[derive(Clone)]
pub struct ConfirmDialog {
    id: String,
    title: String,
    message: String,
    ok: String,
    cancel: String,
    danger: bool,
    busy: Arc<AtomicBool>,
    live: Live,
    on_confirm: Option<Act>,
    on_cancel: Option<Act>,
}

impl ConfirmDialog {
    /// A dialog with a `title` and a `message` (what will happen), buttons from `labels`.
    pub fn new(labels: &Labels, title: &str, message: &str) -> Self {
        Self {
            id: "confirm-dialog".to_owned(),
            title: title.to_owned(),
            message: message.to_owned(),
            ok: labels.ok.clone(),
            cancel: labels.cancel.clone(),
            danger: false,
            busy: Arc::new(AtomicBool::new(false)),
            live: Live::new(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// The prefix of the ids of the parts (several dialogs in one window).
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The word on the confirm button ("Remove").
    #[must_use]
    pub fn confirm_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.ok);
        self
    }

    /// The word on the cancel button ("Keep").
    #[must_use]
    pub fn cancel_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.cancel);
        self
    }

    /// Mark the question as destructive.
    #[must_use]
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Call `handler` on the confirm button (or Enter, unless the dialog is a `danger` one).
    #[must_use]
    pub fn on_confirm(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on the cancel button or Escape.
    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    /// Show that the answer is being carried out: both buttons are off and Escape does nothing until `busy` is false again.
    pub fn set_busy(&self, busy: bool) {
        self.busy.store(busy, Ordering::Relaxed);
        self.live.refresh_now();
    }

    /// Whether the answer is being carried out.
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }

    /// Answer yes. False (nothing happens) while busy.
    pub fn confirm(&self) -> bool {
        if self.is_busy() {
            return false;
        }
        if let Some(handler) = &self.on_confirm {
            handler();
        }
        true
    }

    /// Answer no. False (nothing happens) while busy.
    pub fn cancel(&self) -> bool {
        if self.is_busy() {
            return false;
        }
        if let Some(handler) = &self.on_cancel {
            handler();
        }
        true
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.region("confirm", move || this.render())
    }

    fn render(&self) -> blinc_layout::div::Div {
        let (yes, no) = (self.clone(), self.clone());
        let busy = self.is_busy();
        let variant = if self.danger {
            ButtonVariant::Destructive
        } else {
            ButtonVariant::Primary
        };
        let mut states = Vec::new();
        if self.danger {
            states.push("danger");
        }
        if busy {
            states.push("busy");
        }
        let enter: Option<Act> = if self.danger {
            None
        } else {
            let this = self.clone();
            Some(Arc::new(move || {
                this.confirm();
            }))
        };
        let escape: Act = Arc::new(move || {
            no.cancel();
        });
        let cancel = self.clone();
        frame(Parts {
            id: self.id.clone(),
            kind: "confirm",
            role: "dialog",
            describe: true,
            states,
            title: self.title.clone(),
            body: div().child(text(self.message.clone()).class("fs-dlg-text")),
            footer: vec![
                button(
                    &self.id,
                    "cancel",
                    &self.cancel,
                    ButtonVariant::Outline,
                    busy,
                    Arc::new(move || {
                        cancel.cancel();
                    }),
                ),
                button(
                    &self.id,
                    "ok",
                    &self.ok,
                    variant,
                    busy,
                    Arc::new(move || {
                        yes.confirm();
                    }),
                ),
            ],
            enter,
            escape: Some(escape),
        })
    }
}
