//! The checklist as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the items again through `Live`. The list's
//! own element (keys) and the line for a new item are not inside the live area.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::selector::InputRef;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_primitives::FieldData;

use super::labels::Labels;
use super::model::{Item, Outcome, State};
use crate::Element;
use crate::keys::{KeyEvent, bind_keys};
use crate::live::Live;

type OnChange = Arc<dyn Fn(&[Item]) + Send + Sync>;

/// A checklist. Clones share the state and the callback.
#[derive(Clone)]
pub struct Checklist {
    id: String,
    name: String,
    labels: Labels,
    state: Arc<Mutex<State>>,
    data: SharedTextInputData,
    line: InputRef,
    _keep: Arc<FieldData>,
    live: Live,
    on_change: Option<OnChange>,
}

impl Checklist {
    /// A list named `name` ("Packing list"). Ids: `<id>`, `<id>-item-<key>`, `<id>-remove-<key>`, `<id>-add` (the line).
    pub fn new(name: &str, labels: &Labels, items: Vec<Item>) -> Self {
        let keep = FieldData::new();
        Self {
            id: "checklist".to_owned(),
            name: name.to_owned(),
            labels: labels.clone(),
            state: Arc::new(Mutex::new(State::new(items))),
            data: keep.text_input(),
            line: InputRef::new(),
            _keep: Arc::new(keep),
            live: Live::new(),
            on_change: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Call `handler` with the items after every change by the person (ticked, added, removed, put in another place).
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(&[Item]) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The items now.
    pub fn items(&self) -> Vec<Item> {
        self.slot().items().to_vec()
    }

    /// Add an item as the line does; the key of the new item, `None` for a blank.
    pub fn add(&self, text: &str) -> Option<String> {
        let key = self.slot().add(text)?;
        self.changed();
        Some(key)
    }

    fn changed(&self) {
        self.live.refresh_now();
        if let Some(handler) = &self.on_change {
            handler(&self.items());
        }
    }

    /// A press of the keyboard on the list.
    pub fn press(&self, event: KeyEvent) -> bool {
        let outcome = self.slot().press(event);
        if outcome == Outcome::Ignored {
            return false;
        }
        if outcome == Outcome::Moved {
            self.live.refresh_now();
        } else {
            self.changed();
        }
        true
    }

    fn click_item(&self, key: &str) {
        let done = {
            let mut s = self.slot();
            s.go(key);
            s.toggle(key)
        };
        if done.is_some() {
            self.changed();
        }
    }

    fn click_remove(&self, key: &str) {
        if self.slot().remove(key) {
            self.changed();
        }
    }

    fn submitted(&self) {
        let now = self
            .data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .value
            .clone();
        if self.add(&now).is_some() || now.trim().is_empty() {
            self.line.set_value(String::new());
        }
    }

    /// The list and the line for a new item, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (keys, inner, entering) = (self.clone(), self.clone(), self.clone());
        let list = bind_keys(
            div()
                .id(&self.id)
                .class("fs-checklist")
                .attr("role", "group")
                .attr("aria-label", &self.name),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
        .child(self.live.region("items", move || inner.rows()));
        let line = fs_components_primitives::with_field_font(text_input(&self.data))
            .bind(&self.line)
            .id(&format!("{}-add", self.id))
            .placeholder(&self.labels.add)
            .w_full()
            .on_submit(move |_shift: bool| entering.submitted());
        div()
            .flex_col()
            .gap_px(8.0)
            .w(360.0)
            .child(list)
            .child(div().class("fs-check-add-box").child(line))
    }

    fn rows(&self) -> Div {
        let state = self.slot().clone();
        let mut column = div().flex_col();
        for (i, item) in state.items().iter().enumerate() {
            let mut row = div()
                .id(format!("{}-item-{}", self.id, item.key))
                .class("fs-check-item")
                .attr("role", "checkbox")
                .attr("aria-checked", if item.done { "true" } else { "false" })
                .attr("aria-label", &item.text);
            if item.done {
                row = row.class("fs-check-item-done");
            }
            if state.cursor() == i {
                row = row.class("fs-check-item-current");
            }
            let (pick, drop, key, gone) = (
                self.clone(),
                self.clone(),
                item.key.clone(),
                item.key.clone(),
            );
            column = column.child(
                row.on_click(move |_| pick.click_item(&key))
                    .child(
                        text(if item.done { "\u{2611}" } else { "\u{2610}" }).class("fs-check-box"),
                    )
                    .child(
                        div()
                            .flex_grow_value(1.0)
                            .child(text(item.text.clone()).class("fs-check-text")),
                    )
                    .child(
                        div()
                            .id(format!("{}-remove-{}", self.id, item.key))
                            .class("fs-check-remove")
                            .attr("role", "button")
                            .attr(
                                "aria-label",
                                format!("{} {}", self.labels.remove, item.text),
                            )
                            .on_click(move |_| drop.click_remove(&gone))
                            .child(text("\u{00D7}").class("fs-check-remove-mark")),
                    ),
            );
        }
        column
    }
}
