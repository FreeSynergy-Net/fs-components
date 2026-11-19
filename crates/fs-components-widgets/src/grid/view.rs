//! The grid as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the rows again through `Live` (the grid's own
//! element stays, so the keyboard stays on it).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Column, Outcome, Row, Shown, State};
use crate::Element;
use crate::keys::{Handled, KeyEvent, bind_keys};
use crate::live::Live;

type OnActivate = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// How wide a column is unless told otherwise, in pixels.
const COLUMN_WIDTH: f32 = 140.0;
/// How far a level of a treegrid is indented, in pixels.
const INDENT: f32 = 16.0;

/// A grid or a treegrid. Clones share the state and the callback.
#[derive(Clone)]
pub struct Grid {
    id: String,
    name: Option<String>,
    width: f32,
    state: Arc<Mutex<State>>,
    live: Live,
    on_activate: Option<OnActivate>,
}

impl Grid {
    /// A grid of these columns and rows (child rows are ignored). Ids: `<id>`, `<id>-row-<key>`, `<id>-cell-<key>-<column>`.
    pub fn new(cols: Vec<Column>, rows: &[Row]) -> Self {
        Self::with(State::new(cols, rows, false))
    }

    /// A treegrid: rows with child rows open and close.
    pub fn treegrid(cols: Vec<Column>, rows: &[Row]) -> Self {
        Self::with(State::new(cols, rows, true))
    }

    fn with(state: State) -> Self {
        Self {
            id: "grid".to_owned(),
            name: None,
            width: COLUMN_WIDTH,
            state: Arc::new(Mutex::new(state)),
            live: Live::new(),
            on_activate: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the grid is called (`aria-label`).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// How wide a column is, in pixels.
    #[must_use]
    pub fn column_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// How many rows Page Up and Page Down move.
    #[must_use]
    pub fn page(self, rows: usize) -> Self {
        self.slot().set_page(rows);
        self
    }

    /// Call `handler` with the key of the row and of the column of the cell the person activates (Enter, Space, click).
    #[must_use]
    pub fn on_activate(mut self, handler: impl Fn(&str, &str) + Send + Sync + 'static) -> Self {
        self.on_activate = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The cell the cursor is on: the key of its row and of its column.
    pub fn cursor(&self) -> Option<(String, String)> {
        let state = self.slot();
        let (row, col) = state.cursor();
        let shown = state.shown();
        let at = shown.get(row)?;
        Some((
            state.key_of(at.node).to_owned(),
            state.col_key(col).to_owned(),
        ))
    }

    /// Whether the row with this key is open (a treegrid parent).
    pub fn is_open(&self, key: &str) -> bool {
        let state = self.slot();
        state.find(key).is_some_and(|n| state.is_open(n))
    }

    /// Open or close the row with this key from the program.
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
        let (outcome, told) = {
            let mut state = self.slot();
            let outcome = state.press(event);
            let told = match outcome {
                Outcome::Activated(n, c) => {
                    Some((state.key_of(n).to_owned(), state.col_key(c).to_owned()))
                }
                _ => None,
            };
            (outcome, told)
        };
        if outcome == Outcome::Ignored {
            return false;
        }
        self.live.refresh_now();
        self.said(told);
        true
    }

    fn click(&self, node: usize, col: usize) {
        let told = {
            let mut state = self.slot();
            state
                .go(node, col)
                .then(|| (state.key_of(node).to_owned(), state.col_key(col).to_owned()))
        };
        self.live.refresh_now();
        self.said(told);
    }

    fn said(&self, told: Option<(String, String)>) {
        if let (Some((row, col)), Some(handler)) = (told, &self.on_activate) {
            handler(&row, &col);
        }
    }

    /// The grid, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let tree = self.slot().is_tree();
        let mut root = div()
            .id(&self.id)
            .class("fs-grid")
            .attr("role", if tree { "treegrid" } else { "grid" });
        if tree {
            root = root.class("fs-treegrid");
        }
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("rows", move || inner.rows())),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
    }

    fn rows(&self) -> Div {
        let state = self.slot();
        let shown = state.shown();
        let mut table = div()
            .id(format!("{}-table", self.id))
            .flex_col()
            .attr("aria-rowcount", (shown.len() + 1).to_string())
            .attr("aria-colcount", state.cols().len().to_string());
        let mut head = div()
            .id(format!("{}-head", self.id))
            .class("fs-grid-head")
            .attr("role", "row")
            .attr("aria-rowindex", "1");
        for (c, col) in state.cols().iter().enumerate() {
            head = head.child(
                div()
                    .id(format!("{}-headcell-{}", self.id, col.key))
                    .class("fs-grid-headcell")
                    .attr("role", "columnheader")
                    .attr("aria-colindex", (c + 1).to_string())
                    .w(self.width)
                    .child(text(col.label.clone()).class("fs-grid-headcell-text")),
            );
        }
        table = table.child(head);
        for (r, row) in shown.iter().enumerate() {
            table = table.child(self.row(&state, row, r));
        }
        table
    }

    fn row(&self, state: &State, row: &Shown, index: usize) -> Div {
        let key = state.key_of(row.node);
        let mut line = div()
            .id(format!("{}-row-{key}", self.id))
            .class("fs-grid-row")
            .attr("role", "row")
            .attr("aria-rowindex", (index + 2).to_string());
        if state.is_tree() {
            line = line
                .attr("aria-level", row.level.to_string())
                .attr("aria-setsize", row.set_size.to_string())
                .attr("aria-posinset", row.pos.to_string());
            if let Some(open) = row.expanded {
                line = line.attr("aria-expanded", if open { "true" } else { "false" });
                if open {
                    line = line.class("fs-grid-row-open");
                }
            }
        }
        for c in 0..state.cols().len() {
            let words = state.cells_of(row.node).get(c).cloned().unwrap_or_default();
            let mut cell = div()
                .id(format!("{}-cell-{key}-{}", self.id, state.col_key(c)))
                .class("fs-grid-cell")
                .attr("role", "gridcell")
                .attr("aria-colindex", (c + 1).to_string())
                .w(self.width);
            if state.cursor() == (index, c) {
                cell = cell.class("fs-grid-cell-current");
            }
            if state.is_tree() && c == 0 {
                let mark = match row.expanded {
                    Some(true) => "\u{25BE} ",
                    Some(false) => "\u{25B8} ",
                    None => "",
                };
                #[allow(clippy::cast_precision_loss)]
                let indent = INDENT * (row.level - 1) as f32;
                cell = cell.child(
                    div()
                        .flex_row()
                        .child(div().w(indent).h(1.0).attr("aria-hidden", "true"))
                        .child(
                            div()
                                .class("fs-grid-mark")
                                .attr("aria-hidden", "true")
                                .child(text(mark)),
                        )
                        .child(text(words).class("fs-grid-cell-text")),
                );
            } else {
                cell = cell.child(text(words).class("fs-grid-cell-text"));
            }
            let (this, node) = (self.clone(), row.node);
            line = line.child(cell.on_click(move |_| this.click(node, c)));
        }
        line
    }
}
