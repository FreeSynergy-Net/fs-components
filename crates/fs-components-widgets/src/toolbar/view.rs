//! The toolbar as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the items again through `Live` (the bar's
//! own element stays, so the keyboard stays on it). The pointer and the keys end in the same callback.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Item, Kind, Outcome, State};
use crate::Element;
use crate::keys::{Handled, Key, KeyEvent, bind_keys};
use crate::live::Live;
use crate::roving::Axis;

type OnPress = Arc<dyn Fn(&str, bool) + Send + Sync>;

/// A toolbar. Clones share the state and the callback.
#[derive(Clone)]
pub struct Toolbar {
    id: String,
    name: Option<String>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_press: Option<OnPress>,
}

impl Toolbar {
    /// A bar of these items; ids: `<id>` and `<id>-item-<key>`.
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            id: "toolbar".to_owned(),
            name: None,
            state: Arc::new(Mutex::new(State::new(items, Axis::Horizontal))),
            live: Live::new(),
            on_press: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the bar is called (`aria-label`): "Text formatting". APG: a toolbar with a name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// The items stand one above the other: Up and Down move.
    #[must_use]
    pub fn vertical(self) -> Self {
        self.slot().set_axis(Axis::Vertical);
        self
    }

    /// Call `handler` with the key of the item pressed and whether a toggle is on now (`false` for a button).
    #[must_use]
    pub fn on_press(mut self, handler: impl Fn(&str, bool) + Send + Sync + 'static) -> Self {
        self.on_press = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the toggle with this key is on.
    pub fn is_pressed(&self, key: &str) -> bool {
        self.slot()
            .items()
            .iter()
            .any(|i| i.key == key && i.pressed)
    }

    /// A press of the keyboard.
    pub fn press(&self, key: Key) -> Handled {
        let outcome = self.slot().press(key);
        match outcome {
            Outcome::Ignored => false,
            Outcome::Moved(_) => {
                self.live.refresh_now();
                true
            }
            Outcome::Pressed(i) => {
                self.fired(i);
                true
            }
        }
    }

    fn click(&self, key: &str) {
        let hit = self.slot().press_key(key);
        if let Some(i) = hit {
            self.fired(i);
        }
    }

    fn fired(&self, i: usize) {
        let (key, on) = {
            let state = self.slot();
            let item = &state.items()[i];
            (item.key.clone(), item.kind == Kind::Toggle && item.pressed)
        };
        self.live.refresh_now();
        if let Some(handler) = &self.on_press {
            handler(&key, on);
        }
    }

    /// The bar, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let vertical = self.slot().axis() == Axis::Vertical;
        let mut root = div()
            .id(&self.id)
            .class("fs-toolbar")
            .attr("role", "toolbar")
            .attr(
                "aria-orientation",
                if vertical { "vertical" } else { "horizontal" },
            );
        if vertical {
            root = root.class("fs-toolbar-vertical");
        }
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("items", move || inner.row())),
            Arc::new(move |event: KeyEvent| keys.press(event.key)),
        )
    }

    fn row(&self) -> Div {
        let (items, cursor, vertical) = {
            let state = self.slot();
            (
                state.items().to_vec(),
                state.cursor(),
                state.axis() == Axis::Vertical,
            )
        };
        let mut row = if vertical {
            div().flex_col()
        } else {
            div().flex_row()
        };
        for (i, item) in items.iter().enumerate() {
            if item.kind == Kind::Separator {
                row = row.child(
                    div()
                        .id(format!("{}-sep-{i}", self.id))
                        .class("fs-tool-separator")
                        .attr("role", "separator")
                        .attr(
                            "aria-orientation",
                            if vertical { "horizontal" } else { "vertical" },
                        ),
                );
                continue;
            }
            let mut one = div()
                .id(format!("{}-item-{}", self.id, item.key))
                .class("fs-tool")
                .attr("role", "button")
                .attr("aria-label", &item.label);
            if item.kind == Kind::Toggle {
                one = one.attr("aria-pressed", if item.pressed { "true" } else { "false" });
                if item.pressed {
                    one = one.class("fs-tool-on");
                }
            }
            if cursor == Some(i) {
                one = one.class("fs-tool-current");
            }
            if item.disabled {
                one = one.class("fs-tool-disabled").attr("aria-disabled", "true");
            } else {
                let (this, key) = (self.clone(), item.key.clone());
                one = one.on_click(move |_| this.click(&key));
            }
            row = row.child(one.child(text(item.label.clone()).class("fs-tool-label")));
        }
        row
    }
}
