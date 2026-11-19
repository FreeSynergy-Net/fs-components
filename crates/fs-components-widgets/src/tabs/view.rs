//! The tabs as elements and their handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the row and the panel again through `Live`
//! (the row's own element stays, so the keyboard stays on it). The pointer and the keys end in the same `Tabs::apply`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Outcome, State, Tab};
use crate::Element;
use crate::keys::{Handled, Key, KeyEvent, bind_keys};
use crate::live::Live;
use crate::roving::Axis;

type OnSelect = Arc<dyn Fn(&str) + Send + Sync>;

/// A row of tabs. Clones share the state and the callback.
#[derive(Clone)]
pub struct Tabs {
    id: String,
    name: Option<String>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_select: Option<OnSelect>,
}

impl Tabs {
    /// A row of these tabs; the first one that is not disabled is chosen. Ids: `<id>`, `<id>-tab-<key>`, `<id>-panel-<key>`.
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            id: "tabs".to_owned(),
            name: None,
            state: Arc::new(Mutex::new(State::new(tabs, false, Axis::Horizontal))),
            live: Live::new(),
            on_select: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the row is called (`aria-label`): "Settings sections".
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// The tabs stand one above the other: Up and Down move.
    #[must_use]
    pub fn vertical(self) -> Self {
        self.slot().set_axis(Axis::Vertical);
        self
    }

    /// The manual way: the cursor moves with the arrows, Enter or Space chooses (for tabs whose panel is costly to show).
    #[must_use]
    pub fn manual(self) -> Self {
        self.slot().set_manual(true);
        self
    }

    /// Call `handler` with the key of the tab that was chosen (by pointer or keys, not by [`Tabs::choose`]).
    #[must_use]
    pub fn on_select(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the chosen tab.
    pub fn chosen(&self) -> Option<String> {
        self.slot().chosen_key().map(str::to_owned)
    }

    /// Choose the tab with this key from the program; the row and the panel are drawn again; `false` when there is no such tab.
    pub fn choose(&self, key: &str) -> bool {
        let done = self.slot().choose(key);
        if done {
            self.live.refresh_now();
        }
        done
    }

    /// A press of the keyboard.
    pub fn press(&self, key: Key) -> Handled {
        let (outcome, key_of) = {
            let mut state = self.slot();
            let outcome = state.press(key);
            let key_of = match outcome {
                Outcome::Chosen(i) => Some(state.tabs()[i].key.clone()),
                _ => None,
            };
            (outcome, key_of)
        };
        if outcome == Outcome::Ignored {
            return false;
        }
        self.live.refresh_now();
        if let (Some(key), Some(handler)) = (key_of, &self.on_select) {
            handler(&key);
        }
        true
    }

    fn click(&self, key: &str) {
        let changed = {
            let mut state = self.slot();
            let before = state.chosen();
            state.choose(key) && state.chosen() != before
        };
        self.live.refresh_now();
        if let (true, Some(handler)) = (changed, &self.on_select) {
            handler(key);
        }
    }

    /// The row of tabs, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let vertical = self.slot().axis() == Axis::Vertical;
        let mut root = div()
            .id(&self.id)
            .class("fs-tabs")
            .attr("role", "tablist")
            .attr(
                "aria-orientation",
                if vertical { "vertical" } else { "horizontal" },
            );
        if vertical {
            root = root.class("fs-tabs-vertical");
        }
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("tabs", move || inner.row())),
            Arc::new(move |event: KeyEvent| keys.press(event.key)),
        )
    }

    fn row(&self) -> Div {
        let (tabs, chosen, cursor, vertical) = {
            let state = self.slot();
            (
                state.tabs().to_vec(),
                state.chosen(),
                state.cursor(),
                state.axis() == Axis::Vertical,
            )
        };
        let mut row = if vertical { div().flex_col() } else { div().flex_row() };
        for (i, tab) in tabs.iter().enumerate() {
            let selected = chosen == Some(i);
            let mut one = div()
                .id(format!("{}-tab-{}", self.id, tab.key))
                .class("fs-tab")
                .attr("role", "tab")
                .attr("aria-selected", if selected { "true" } else { "false" })
                .attr("aria-controls", format!("{}-panel-{}", self.id, tab.key));
            if selected {
                one = one.class("fs-tab-selected");
            }
            if cursor == Some(i) && !selected {
                one = one.class("fs-tab-current");
            }
            if tab.disabled {
                one = one.class("fs-tab-disabled").attr("aria-disabled", "true");
            } else {
                let (this, key) = (self.clone(), tab.key.clone());
                one = one.on_click(move |_| this.click(&key));
            }
            row = row.child(one.child(text(tab.label.clone()).class("fs-tab-label")));
        }
        row
    }

    /// The panel of the chosen tab: `content` is given the key of the chosen tab and fills the panel. It is drawn again when the
    /// choice changes. Role `tabpanel`, named by its tab.
    pub fn panel<F>(&self, content: F) -> impl Element + use<F>
    where
        F: Fn(&str) -> Div + Send + Sync + 'static,
    {
        let this = self.clone();
        self.live.clone().region("panel", move || {
            let key = this.chosen().unwrap_or_default();
            div()
                .id(format!("{}-panel-{key}", this.id))
                .class("fs-tabpanel")
                .attr("role", "tabpanel")
                .attr("aria-labelledby", format!("{}-tab-{key}", this.id))
                .child(content(&key))
        })
    }
}
