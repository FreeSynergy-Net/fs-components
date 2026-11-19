//! The chip row as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the chips again through `Live`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::model::{Chip, Outcome, State};
use crate::Element;
use crate::keys::{KeyEvent, bind_keys};
use crate::live::Live;

type OnRemove = Arc<dyn Fn(&str) + Send + Sync>;

/// A row of chips. Clones share the state and the callback.
#[derive(Clone)]
pub struct ChipList {
    id: String,
    name: String,
    labels: Labels,
    state: Arc<Mutex<State>>,
    live: Live,
    on_remove: Option<OnRemove>,
}

impl ChipList {
    /// A row named `name` ("Keywords"). Ids: `<id>`, `<id>-chip-<key>`, `<id>-remove-<key>`.
    pub fn new(name: &str, labels: &Labels, chips: Vec<Chip>) -> Self {
        Self {
            id: "chips".to_owned(),
            name: name.to_owned(),
            labels: labels.clone(),
            state: Arc::new(Mutex::new(State::new(chips))),
            live: Live::new(),
            on_remove: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Call `handler` with the key of a chip the person removed (Delete, Backspace or the remove button).
    #[must_use]
    pub fn on_remove(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_remove = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The keys of the chips, in order.
    pub fn keys(&self) -> Vec<String> {
        self.slot().chips().iter().map(|c| c.key.clone()).collect()
    }

    /// Add a chip from the program; false when the key is there.
    pub fn add(&self, chip: Chip) -> bool {
        let done = self.slot().add(chip);
        if done {
            self.live.refresh_now();
        }
        done
    }

    fn removed(&self, key: &str) {
        self.live.refresh_now();
        if let Some(handler) = &self.on_remove {
            handler(key);
        }
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> bool {
        let outcome = self.slot().press(event.key);
        match outcome {
            Outcome::Ignored => false,
            Outcome::Moved => {
                self.live.refresh_now();
                true
            }
            Outcome::Removed(key) => {
                self.removed(&key);
                true
            }
        }
    }

    fn click_remove(&self, key: &str) {
        if self.slot().remove(key) {
            self.removed(key);
        }
    }

    /// The row, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (keys, inner) = (self.clone(), self.clone());
        bind_keys(
            div()
                .id(&self.id)
                .class("fs-chips")
                .attr("role", "list")
                .attr("aria-label", &self.name),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
        .child(self.live.region("chips", move || inner.row()))
    }

    fn row(&self) -> Div {
        let state = self.slot().clone();
        let mut row = div().flex_row().flex_wrap().gap_px(8.0);
        for (i, chip) in state.chips().iter().enumerate() {
            let mut one = div()
                .id(format!("{}-chip-{}", self.id, chip.key))
                .class("fs-chip")
                .attr("role", "listitem");
            if state.cursor() == i {
                one = one.class("fs-chip-current");
            }
            let this = self.clone();
            let key = chip.key.clone();
            one = one
                .child(text(chip.label.clone()).class("fs-chip-label"))
                .child(
                    div()
                        .id(format!("{}-remove-{}", self.id, chip.key))
                        .class("fs-chip-remove")
                        .attr("role", "button")
                        .attr(
                            "aria-label",
                            format!("{} {}", self.labels.remove, chip.label),
                        )
                        .on_click(move |_| this.click_remove(&key))
                        .child(text("\u{00D7}").class("fs-chip-remove-mark")),
                );
            row = row.child(one);
        }
        row
    }
}
