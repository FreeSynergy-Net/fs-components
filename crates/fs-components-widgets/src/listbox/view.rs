//! The listbox as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the options again through `Live` (the list's
//! own element stays, so the keyboard stays on it). The pointer and the keys end in the same callback.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Opt, Outcome, State};
use crate::Element;
use crate::keys::{Handled, KeyEvent, bind_keys};
use crate::live::Live;

type OnChange = Arc<dyn Fn(&[String]) + Send + Sync>;

/// A listbox. Clones share the state and the callback.
#[derive(Clone)]
pub struct Listbox {
    id: String,
    name: Option<String>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_change: Option<OnChange>,
}

impl Listbox {
    /// A list of these options; one may be picked at a time (the first that can be). Ids: `<id>`, `<id>-opt-<key>`.
    pub fn new(opts: Vec<Opt>) -> Self {
        Self::with(opts, false)
    }

    /// A list of these options of which several may be picked (`aria-multiselectable`); none is picked at first.
    pub fn multiple(opts: Vec<Opt>) -> Self {
        Self::with(opts, true)
    }

    fn with(opts: Vec<Opt>, multi: bool) -> Self {
        Self {
            id: "listbox".to_owned(),
            name: None,
            state: Arc::new(Mutex::new(State::new(opts, multi))),
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

    /// What the list is called (`aria-label`): "Language".
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Call `handler` with the keys of the picked options after every change made by the person.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(&[String]) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The keys of the picked options.
    pub fn picked(&self) -> Vec<String> {
        self.slot().picked_keys()
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> Handled {
        let outcome = self.slot().press(event);
        match outcome {
            Outcome::Ignored => false,
            Outcome::Moved(_) => {
                self.live.refresh_now();
                true
            }
            Outcome::Picked => {
                self.changed();
                true
            }
        }
    }

    fn click(&self, key: &str) {
        if self.slot().pick_key(key) {
            self.changed();
        }
    }

    fn changed(&self) {
        let keys = self.picked();
        self.live.refresh_now();
        if let Some(handler) = &self.on_change {
            handler(&keys);
        }
    }

    /// The list, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let mut root = div()
            .id(&self.id)
            .class("fs-listbox")
            .attr("role", "listbox");
        if self.slot().multi() {
            root = root.attr("aria-multiselectable", "true");
        }
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("options", move || inner.rows())),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
    }

    fn rows(&self) -> Div {
        let state = self.slot().clone();
        let mut column = div().flex_col();
        for (i, opt) in state.opts().iter().enumerate() {
            let picked = state.is_picked(i);
            let mut one = div()
                .id(format!("{}-opt-{}", self.id, opt.key))
                .class("fs-option")
                .attr("role", "option")
                .attr("aria-selected", if picked { "true" } else { "false" });
            if picked {
                one = one.class("fs-option-picked");
            }
            if state.cursor() == Some(i) {
                one = one.class("fs-option-current");
            }
            if opt.disabled {
                one = one.class("fs-option-disabled").attr("aria-disabled", "true");
            } else {
                let (this, key) = (self.clone(), opt.key.clone());
                one = one.on_click(move |_| this.click(&key));
            }
            column = column.child(one.child(text(opt.label.clone()).class("fs-option-label")));
        }
        column
    }
}
