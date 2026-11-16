//! `choice-dialog`: one thing out of a few ("which of these?"), with a button to take it and one to leave.
//!
//! Pattern: State (the picked option, pure) + Command (`choose`, `cancel`) + Observer (`Live` draws the options again). The
//! list is ONE tab stop (WAI-ARIA radio group): Up and Down (also Left and Right) move the pick, Home and End jump, Enter takes
//! it; a disabled option is skipped by the keys and ignores the pointer. The options are drawn again inside the list, the list
//! itself is not, so a key press never loses the keyboard.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::choice_option::ChoiceOption;
use super::frame::{Act, Parts, button, frame};
use super::labels::Labels;
use crate::Element;
use crate::keys::{Key, bind_keys};
use crate::live::Live;

/// Called with the key of the picked option.
type OnChoose = Arc<dyn Fn(&str) + Send + Sync>;

/// The dialog. Clones share the pick and the callbacks.
#[derive(Clone)]
pub struct ChoiceDialog {
    id: String,
    title: String,
    message: Option<String>,
    options: Arc<Vec<ChoiceOption>>,
    picked: Arc<Mutex<Option<usize>>>,
    ok: String,
    cancel: String,
    live: Live,
    on_choose: Option<OnChoose>,
    on_cancel: Option<Act>,
}

impl ChoiceDialog {
    /// A dialog with a `title`, buttons from `labels`; add the options with [`ChoiceDialog::option`].
    pub fn new(labels: &Labels, title: &str) -> Self {
        Self {
            id: "choice-dialog".to_owned(),
            title: title.to_owned(),
            message: None,
            options: Arc::new(Vec::new()),
            picked: Arc::new(Mutex::new(None)),
            ok: labels.ok.clone(),
            cancel: labels.cancel.clone(),
            live: Live::new(),
            on_choose: None,
            on_cancel: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A sentence above the options.
    #[must_use]
    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_owned());
        self
    }

    /// Add an option. The first one that can be picked is picked at the start.
    #[must_use]
    pub fn option(mut self, option: ChoiceOption) -> Self {
        Arc::make_mut(&mut self.options).push(option);
        let first = self.options.iter().position(|o| !o.disabled);
        *self.pick() = first;
        self
    }

    /// The word on the take button ("Install").
    #[must_use]
    pub fn choose_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.ok);
        self
    }

    /// The word on the leave button ("Not now").
    #[must_use]
    pub fn cancel_label(mut self, word: &str) -> Self {
        word.clone_into(&mut self.cancel);
        self
    }

    /// Call `handler` with the key of the picked option.
    #[must_use]
    pub fn on_choose(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_choose = Some(Arc::new(handler));
        self
    }

    /// Call `handler` on the cancel button or Escape.
    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    fn pick(&self) -> std::sync::MutexGuard<'_, Option<usize>> {
        self.picked.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the picked option.
    pub fn picked(&self) -> Option<String> {
        let i = (*self.pick())?;
        self.options.get(i).map(|o| o.key.clone())
    }

    /// Pick the option with this key; false when there is none or it is disabled.
    pub fn select(&self, key: &str) -> bool {
        let Some(i) = self
            .options
            .iter()
            .position(|o| o.key == key && !o.disabled)
        else {
            return false;
        };
        *self.pick() = Some(i);
        self.live.refresh_now();
        true
    }

    /// Move the pick by `delta` options (negative: up), over the options that can be picked, without wrapping.
    pub fn step(&self, delta: isize) {
        let now = *self.pick();
        let pickable: Vec<usize> = (0..self.options.len())
            .filter(|&i| !self.options[i].disabled)
            .collect();
        let at = now.and_then(|n| pickable.iter().position(|&i| i == n));
        let to = match at {
            Some(a) => a
                .saturating_add_signed(delta)
                .min(pickable.len().saturating_sub(1)),
            None => 0,
        };
        if let Some(&i) = pickable.get(to) {
            *self.pick() = Some(i);
            self.live.refresh_now();
        }
    }

    /// Take the pick. False (nothing happens) when nothing is picked.
    pub fn choose(&self) -> bool {
        let Some(key) = self.picked() else {
            return false;
        };
        if let Some(handler) = &self.on_choose {
            handler(&key);
        }
        true
    }

    /// Leave without a choice.
    pub fn cancel(&self) {
        if let Some(handler) = &self.on_cancel {
            handler();
        }
    }

    fn keys(&self, event: &crate::keys::KeyEvent) -> bool {
        match event.key {
            Key::Up | Key::Left => self.step(-1),
            Key::Down | Key::Right => self.step(1),
            Key::Home => self.step(isize::MIN / 2),
            Key::End => self.step(isize::MAX / 2),
            Key::Activate | Key::Enter => return self.choose(),
            _ => return false,
        }
        true
    }

    /// The element to place in a modal.
    pub fn build(&self) -> impl Element + use<> {
        let (ok, no, enter, esc) = (self.clone(), self.clone(), self.clone(), self.clone());
        let mut body = div().class("fs-ch");
        if let Some(message) = &self.message {
            body = body.child(text(message.clone()).class("fs-dlg-text"));
        }
        let inner = self.clone();
        let list = div()
            .id(format!("{}-list", self.id))
            .class("fs-ch-list")
            .attr("role", "radiogroup");
        let this = self.clone();
        let list = bind_keys(
            list.child(inner.live.region("options", move || inner.options_view())),
            Arc::new(move |event| this.keys(&event)),
        );
        let nothing = self.picked().is_none();
        frame(Parts {
            id: self.id.clone(),
            kind: "choice",
            states: Vec::new(),
            title: self.title.clone(),
            body: body.child(list),
            footer: vec![
                button(
                    &self.id,
                    "cancel",
                    &self.cancel,
                    ButtonVariant::Outline,
                    false,
                    Arc::new(move || no.cancel()),
                ),
                button(
                    &self.id,
                    "ok",
                    &self.ok,
                    ButtonVariant::Primary,
                    nothing,
                    Arc::new(move || {
                        ok.choose();
                    }),
                ),
            ],
            enter: Some(Arc::new(move || {
                enter.choose();
            })),
            escape: Some(Arc::new(move || esc.cancel())),
        })
    }

    fn options_view(&self) -> Div {
        let at = *self.pick();
        let mut col = div().class("fs-ch-options");
        for (i, option) in self.options.iter().enumerate() {
            let mut row = div()
                .id(format!("{}-opt-{}", self.id, option.key))
                .class("fs-ch-opt")
                .attr("role", "radio");
            let on = at == Some(i);
            if on {
                row = row.class("fs-ch-opt-on");
            }
            if option.disabled {
                row = row.class("fs-ch-opt-off");
            }
            let mut mark = div()
                .id(format!("{}-mark-{}", self.id, option.key))
                .class("fs-ch-mark");
            if on {
                mark = mark.child(div().class("fs-ch-dot"));
            }
            let mut words = div()
                .class("fs-ch-words")
                .child(text(option.label.clone()).class("fs-ch-label"));
            if let Some(hint) = &option.hint {
                words = words.child(text(hint.clone()).class("fs-ch-hint"));
            }
            let (this, key) = (self.clone(), option.key.clone());
            row = row.child(mark).child(words).on_click(move |_| {
                this.select(&key);
            });
            col = col.child(row);
        }
        col
    }
}
