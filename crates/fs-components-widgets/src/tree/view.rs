//! The tree as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the items again through `Live` (the tree's
//! own element stays, so the keyboard stays on it).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Outcome, State};
use crate::Element;
use crate::keys::{Handled, KeyEvent, bind_keys};
use crate::live::Live;
use crate::tree_view::TreeNode;

type OnPick = Arc<dyn Fn(&str) + Send + Sync>;

/// How far a level is indented, in pixels.
const INDENT: f32 = 16.0;

/// A tree. Clones share the state and the callback.
#[derive(Clone)]
pub struct Tree {
    id: String,
    name: Option<String>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_pick: Option<OnPick>,
}

impl Tree {
    /// A tree of these top items. Ids: `<id>`, `<id>-item-<key>`, `<id>-group-<key>`.
    pub fn new(top: &[TreeNode]) -> Self {
        Self {
            id: "tree".to_owned(),
            name: None,
            state: Arc::new(Mutex::new(State::new(top))),
            live: Live::new(),
            on_pick: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the tree is called (`aria-label`): "Folders".
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Call `handler` with the key of the item the person picks (click, Enter, Space).
    #[must_use]
    pub fn on_pick(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_pick = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the picked item.
    pub fn picked(&self) -> Option<String> {
        let state = self.slot();
        state.picked().map(|n| state.key_of(n).to_owned())
    }

    /// Whether the item with this key is open.
    pub fn is_open(&self, key: &str) -> bool {
        let state = self.slot();
        state.find(key).is_some_and(|n| state.is_open(n))
    }

    /// Open or close the item with this key from the program.
    pub fn set_open(&self, key: &str, open: bool) {
        let changed = {
            let mut state = self.slot();
            state.find(key).is_some_and(|n| state.set_open(n, open))
        };
        if changed {
            self.live.refresh_now();
        }
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> Handled {
        let (outcome, key) = {
            let mut state = self.slot();
            let outcome = state.press(event.key);
            let key = match outcome {
                Outcome::Picked(n) => Some(state.key_of(n).to_owned()),
                _ => None,
            };
            (outcome, key)
        };
        if outcome == Outcome::Ignored {
            return false;
        }
        self.live.refresh_now();
        self.said(key);
        true
    }

    fn click(&self, node: usize) {
        let key = {
            let mut state = self.slot();
            state.pick(node);
            state.key_of(node).to_owned()
        };
        self.live.refresh_now();
        self.said(Some(key));
    }

    fn said(&self, key: Option<String>) {
        if let (Some(key), Some(handler)) = (key, &self.on_pick) {
            handler(&key);
        }
    }

    /// The tree, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let mut root = div()
            .id(&self.id)
            .class("fs-tree-root")
            .attr("role", "tree");
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("items", move || inner.items())),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
    }

    fn items(&self) -> Div {
        let state = self.slot();
        self.branch(&state, state.roots(), 1)
    }

    fn branch(&self, state: &State, siblings: &[usize], level: usize) -> Div {
        let mut column = div().flex_col();
        for (p, &node) in siblings.iter().enumerate() {
            let row = state.row(node, level, siblings.len(), p + 1);
            let mut item = self.item(state, &row);
            if row.expanded == Some(true) {
                item = item.child(
                    div()
                        .id(format!("{}-group-{}", self.id, row.key))
                        .class("fs-treeitem-group")
                        .attr("role", "group")
                        .child(self.branch(state, state.children_of(node), level + 1)),
                );
            }
            column = column.child(item);
        }
        column
    }

    /// The item: the `treeitem` carries the roles and numbers; the row inside it is what is drawn, clicked and marked.
    fn item(&self, state: &State, row: &super::model::Row) -> Div {
        let picked = state.picked() == Some(row.node);
        let mut item = div()
            .id(format!("{}-item-{}", self.id, row.key))
            .class("fs-treeitem")
            .attr("role", "treeitem")
            .attr("aria-level", row.level.to_string())
            .attr("aria-setsize", row.set_size.to_string())
            .attr("aria-posinset", row.pos.to_string())
            .attr("aria-selected", if picked { "true" } else { "false" });
        if let Some(open) = row.expanded {
            item = item.attr("aria-expanded", if open { "true" } else { "false" });
        }
        let mut line = div()
            .id(format!("{}-row-{}", self.id, row.key))
            .class("fs-treeitem-row");
        if picked {
            line = line.class("fs-treeitem-picked");
        }
        if state.cursor() == Some(row.node) {
            line = line.class("fs-treeitem-current");
        }
        let mark = match row.expanded {
            Some(true) => "\u{25BE}",
            Some(false) => "\u{25B8}",
            None => " ",
        };
        let (this, node) = (self.clone(), row.node);
        #[allow(clippy::cast_precision_loss)]
        let indent = INDENT * (row.level - 1) as f32;
        item.child(
            line.on_click(move |_| this.click(node))
                .child(div().w(indent).h(1.0).attr("aria-hidden", "true"))
                .child(
                    div()
                        .class("fs-treeitem-mark")
                        .attr("aria-hidden", "true")
                        .child(text(mark)),
                )
                .child(text(row.label.clone()).class("fs-treeitem-label")),
        )
    }
}
