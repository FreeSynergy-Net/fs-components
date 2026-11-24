//! The one line the reader types for the board (a column name, a limit): an `input-dialog` that stays open while the line is
//! refused and says why, and closes when it is taken or cancelled. At most one question is open per board.
//!
//! Pattern: Strategy (the `apply` of the line) inside one reusable dialog.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_widgets::dialogs::{InputDialog, Labels};
use fs_components_widgets::live::Live;

/// The open question to the reader and the area that draws it.
#[derive(Default)]
pub(crate) struct Ask {
    dialog: Mutex<Option<InputDialog>>,
    pub live: Live,
}

impl Ask {
    /// The dialog that is open now.
    pub fn current(&self) -> Option<InputDialog> {
        self.dialog
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn set(&self, dialog: Option<InputDialog>) {
        *self.dialog.lock().unwrap_or_else(PoisonError::into_inner) = dialog;
        self.live.refresh_now();
    }
}

/// What the reader's line does: `Err` (a sentence for the reader) keeps the dialog open.
type Apply = dyn Fn(&str) -> Result<(), String> + Send + Sync;

/// The words of one dialog (already in the reader's language).
pub(crate) struct Words {
    pub id: &'static str,
    pub title: String,
    pub placeholder: String,
    /// A line above the field that says what to type (shown until a refusal replaces it).
    pub hint: Option<String>,
}

/// One text question to the reader.
pub(crate) struct Text {
    ask: Arc<Ask>,
    labels: Labels,
    words: Words,
    apply: Box<Apply>,
}

impl Text {
    pub fn new(
        ask: &Arc<Ask>,
        labels: &Labels,
        words: Words,
        apply: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    ) -> Arc<Self> {
        Arc::new(Self {
            ask: ask.clone(),
            labels: labels.clone(),
            words,
            apply: Box::new(apply),
        })
    }

    /// Open the dialog with `initial` in the field; `refusal` is what the last try was told.
    pub fn open(self: &Arc<Self>, initial: &str, refusal: Option<&str>) {
        let (me, done, left) = (self.clone(), self.ask.clone(), self.ask.clone());
        let w = &self.words;
        let mut dialog = InputDialog::new(&self.labels, &w.title, &w.placeholder)
            .id(w.id)
            .initial(initial);
        let message = refusal.or(w.hint.as_deref());
        if let Some(message) = message {
            dialog = dialog.message(message);
        }
        let dialog = dialog
            .on_input(move |line| match (me.apply)(line) {
                Ok(()) => done.set(None),
                // the refusal keeps the dialog open, with what was typed, so the reader can correct it
                Err(why) => me.open(line, Some(&why)),
            })
            .on_cancel(move || left.set(None));
        self.ask.set(Some(dialog));
    }
}
