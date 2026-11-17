//! The entry line: one line of text; Enter hands it to the host and empties the line.
//!
//! Pattern: Adapter over Blinc's `text_input` (the same field, the same font rule, as the search field); the
//! one thing new is `on_submit`. The text a host receives is trimmed and never empty; a line of blanks is kept as it
//! is. The host keeps one `Entry` per place and builds it again on every drawing, so what is typed survives.
//! The host can put text back into the line ([`Entry::set`], for a create that failed, so nothing typed is lost) and say why
//! in a notice place under it ([`Entry::set_notice`]); the notice goes away at the next submit.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_primitives::{FieldData, with_field_font};

use super::events::{Events, redraw};

/// One entry line.
#[derive(Clone)]
pub struct Entry {
    data: SharedTextInputData,
    _keep: FieldData,
    notice: Arc<Mutex<Option<String>>>,
}

/// What the entry hands over on Enter: the trimmed text, and whether Shift was held.
pub type OnEnter = Arc<dyn Fn(&str, bool) + Send + Sync>;

/// What Enter hands over: the trimmed text, or `None` for a line with no letter in it.
pub fn submitted(typed: &str) -> Option<String> {
    let text = typed.trim();
    (!text.is_empty()).then(|| text.to_string())
}

impl Entry {
    /// A line showing `placeholder` while empty.
    pub fn new(placeholder: &str) -> Self {
        let keep = FieldData::new();
        Self {
            data: keep.text_input_with_placeholder(placeholder),
            _keep: keep,
            notice: Arc::default(),
        }
    }

    /// Put `text` into the line, the cursor at its end (a create that failed gives back what was typed).
    pub fn set(&self, text: &str) {
        if let Ok(mut d) = self.data.lock() {
            d.value = text.to_string();
            d.cursor = text.chars().count();
            d.selection_start = None;
            d.force_sync_once = true;
        }
        redraw();
    }

    /// Say something under the line (`None` takes it away); the line stays as it is. It is the host's sentence, in the
    /// reader's language; the next submit removes it.
    pub fn set_notice(&self, notice: Option<&str>) {
        *self.notice.lock().unwrap_or_else(PoisonError::into_inner) = notice.map(str::to_owned);
        redraw();
    }

    /// The notice under the line now.
    pub fn notice(&self) -> Option<String> {
        self.notice
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// What is typed now.
    pub fn text(&self) -> String {
        self.data
            .lock()
            .map(|d| d.value.clone())
            .unwrap_or_default()
    }

    /// Empty the line.
    pub fn clear(&self) {
        if let Ok(mut d) = self.data.lock() {
            d.value.clear();
            d.cursor = 0;
            d.selection_start = None;
            d.force_sync_once = true;
        }
    }

    /// The element; `place` is handed to the host with the text.
    pub fn build(&self, id: &str, place: &str, events: &Events) -> Div {
        let (enter, place) = (events.enter.clone(), place.to_string());
        self.build_with(
            id,
            Arc::new(move |text: &str, _shift: bool| {
                if let Some(enter) = &enter {
                    enter(&place, text);
                }
            }),
        )
    }

    /// The element; `on_enter` gets the trimmed text and whether Shift was held (a board puts the card at the top then).
    pub fn build_with(&self, id: &str, on_enter: OnEnter) -> Div {
        let me = self.clone();
        let field = with_field_font(text_input(&self.data))
            .class("fs-plan-entry-field")
            .w_full()
            .on_submit(move |shift| {
                let Some(text) = submitted(&me.text()) else {
                    return;
                };
                me.clear();
                me.set_notice(None);
                on_enter(&text, shift);
                redraw();
            });
        let mut root = div()
            .id(id.to_string())
            .class("fs-plan-entry")
            .w_full()
            .child(field);
        if let Some(notice) = self.notice() {
            root = root.child(
                text(notice)
                    .class("fs-plan-entry-notice")
                    .id(format!("{id}-notice")),
            );
        }
        root
    }
}
