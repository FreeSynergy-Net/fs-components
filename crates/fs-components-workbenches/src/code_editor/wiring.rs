//! What the code editor needs of its host and of the form: the record, the draft and the write port, and the three things a person
//! can do (type, save, discard). No drawing here, so the rules can be tested without a window.
//!
//! Pattern: Mediator — one place that joins the form read, the draft and the write port; the drawing asks it and tells it.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::DrawCtx;
use fs_components_core::form_write::{FormWrite, WriteArg};
use fs_components_core::live::Live;

use super::draft::Draft;

/// The channel of the form that writes the whole content (`single-record@1.1.0`, `edit`).
pub(super) const EDIT: &str = "edit";
/// The channel with the text.
const CONTENT: &str = "content";
/// The channel with the media type of the text.
const MEDIA_TYPE: &str = "media-type";
/// The media type taken when the record names none.
const PLAIN: &str = "text/plain";
/// The media types that have a preview.
const PREVIEW: [&str; 1] = ["text/markdown"];

pub(super) struct Wiring {
    input: Mutex<FormInput>,
    draft: Mutex<Draft>,
    write: Arc<dyn FormWrite>,
    reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    say: Arc<dyn Fn(&str) -> String + Send + Sync>,
    refused: Mutex<Option<String>>,
    pub(super) live: Live,
}

fn first(input: &FormInput) -> Option<&FormRow> {
    input.rows.first()
}

fn parts(input: &FormInput) -> (String, String) {
    let row = first(input);
    let get = |channel: &str| row.and_then(|r| r.get(channel)).map(str::to_owned);
    (
        row.and_then(|r| r.all(CONTENT).first().cloned())
            .unwrap_or_default(),
        get(MEDIA_TYPE).unwrap_or_else(|| PLAIN.to_owned()),
    )
}

impl Wiring {
    pub(super) fn new(input: FormInput, ctx: &DrawCtx) -> Self {
        let (content, media_type) = parts(&input);
        Self {
            draft: Mutex::new(Draft::new(&content, &media_type)),
            input: Mutex::new(input),
            write: ctx.write.clone(),
            reread: ctx.reread.clone(),
            say: ctx.say.clone(),
            refused: Mutex::new(None),
            live: Live::new().gap(150),
        }
    }

    pub(super) fn say(&self, key: &str) -> String {
        (self.say)(key)
    }

    pub(super) fn draft(&self) -> MutexGuard<'_, Draft> {
        self.draft.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn input(&self) -> MutexGuard<'_, FormInput> {
        self.input.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the record being edited.
    pub(super) fn key(&self) -> Option<String> {
        first(&self.input()).map(|r| r.key.clone())
    }

    /// The title of the record (`label`).
    pub(super) fn label(&self) -> String {
        first(&self.input())
            .and_then(|r| r.get("label"))
            .unwrap_or_default()
            .to_owned()
    }

    /// The media type of the text, which picks the colours.
    pub(super) fn media_type(&self) -> String {
        parts(&self.input()).1
    }

    /// Has the text a preview (variant `split`)?
    pub(super) fn has_preview(&self) -> bool {
        PREVIEW.contains(&self.media_type().as_str())
    }

    /// Why the text is locked, as a sentence; `None` = it may be edited. The host's answer (`can`) decides; the `state` of the
    /// record may name the reason (`code-editor.readonly.<state>`), else the general one.
    pub(super) fn locked(&self) -> Option<String> {
        let (key, state) = {
            let input = self.input();
            let row = first(&input)?;
            (row.key.clone(), row.get("state").map(str::to_owned))
        };
        if self.write.can(EDIT, Some(&key)) {
            return None;
        }
        let general = "code-editor.readonly.no-edit";
        let named = state.map(|s| format!("code-editor.readonly.{s}"));
        Some(match named {
            Some(k) if self.say(&k) != k => self.say(&k),
            _ => self.say(general),
        })
    }

    /// The last refusal of a save, as a sentence.
    pub(super) fn refusal(&self) -> Option<String> {
        let code = self
            .refused
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()?;
        Some(self.say(&format!("code-editor.refused.{code}")))
    }

    fn set_refused(&self, code: Option<String>) {
        *self.refused.lock().unwrap_or_else(PoisonError::into_inner) = code;
    }

    /// A change of the draft by typing: only the draft (and what shows it) changes, nothing is written.
    pub(super) fn typed(&self) {
        self.set_refused(None);
        self.live.refresh();
    }

    /// Back to what the module holds.
    pub(super) fn discard(&self) {
        self.set_refused(None);
        self.draft().discard();
        self.live.refresh_now();
    }

    /// Save: exactly one `edit` with the whole text, only when there is a change and the host allows it. After an accepted call the
    /// form is read again and the draft follows what the module now says.
    pub(super) fn save(&self) {
        let Some(key) = self.key() else { return };
        let text = {
            let draft = self.draft();
            if !draft.is_dirty() {
                return;
            }
            draft.text()
        };
        if !self.write.can(EDIT, Some(&key)) {
            self.set_refused(Some("readonly".to_owned()));
            return;
        }
        match self
            .write
            .send(EDIT, Some(&key), &[WriteArg::text(CONTENT, &text)])
        {
            Ok(_) => {
                self.set_refused(None);
                self.draft().saved(&text);
                self.load((self.reread)());
            }
            Err(refused) => self.set_refused(Some(refused.code)),
        }
        self.live.refresh_now();
    }

    /// The module says something new: a clean draft follows, a typed one stays.
    pub(super) fn load(&self, input: FormInput) {
        if first(&input).is_some() {
            let (content, media_type) = parts(&input);
            self.draft().adopt(&content, &media_type);
        }
        *self.input() = input;
        self.live.refresh();
    }
}
