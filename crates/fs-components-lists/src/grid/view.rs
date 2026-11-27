//! The grid as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the rows again through `Live` (the grid's own
//! element stays, so the keyboard stays on it).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::div;

use super::cell::{CellView, TextCells};
use super::model::{Column, Outcome, Row, State};
use crate::Element;
use crate::keys::{Handled, KeyEvent, bind_keys};
use crate::live::Live;

type OnActivate = Arc<dyn Fn(&str, &str) + Send + Sync>;
type FirstKey = Arc<dyn Fn(&KeyEvent) -> Handled + Send + Sync>;

/// How wide a column is unless told otherwise, in pixels.
const COLUMN_WIDTH: f32 = 140.0;

/// A grid or a treegrid. Clones share the state and the callback.
#[derive(Clone)]
pub struct Grid {
    pub(super) id: String,
    name: Option<String>,
    pub(super) width: f32,
    pub(super) headless: bool,
    state: Arc<Mutex<State>>,
    live: Live,
    pub(super) view: Arc<dyn CellView>,
    on_activate: Option<OnActivate>,
    on_enter: Option<OnActivate>,
    on_leave: Option<OnActivate>,
    first_key: Option<FirstKey>,
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
            headless: false,
            state: Arc::new(Mutex::new(state)),
            live: Live::new(),
            view: Arc::new(TextCells),
            on_activate: None,
            on_enter: None,
            on_leave: None,
            first_key: None,
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

    /// The variant `headless`: no head row (the cells say themselves what they are, as the tiles of a tile grid do).
    #[must_use]
    pub fn headless(mut self) -> Self {
        self.headless = true;
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

    /// What the cells show: the strategy gives an element per cell and per head cell (`None` = the text). Without it every cell is text.
    #[must_use]
    pub fn cells(mut self, view: Arc<dyn CellView>) -> Self {
        self.view = view;
        self
    }

    /// The cells hold elements and take the keys (variant `cells-hold`, APG interactive cells): Enter on a body cell goes into it,
    /// Escape comes back (the table is in `hold.rs`). Without it the grid is as before.
    #[must_use]
    pub fn hold_cells(self) -> Self {
        self.slot().set_holds(true);
        self
    }

    /// Call `handler` with the keys of the row and the column when the keys go into a cell (Enter, `hold_cells`).
    #[must_use]
    pub fn on_enter(mut self, handler: impl Fn(&str, &str) + Send + Sync + 'static) -> Self {
        self.on_enter = Some(Arc::new(handler));
        self
    }

    /// Call `handler` with the keys of the row and the column when the keys come back out of a cell.
    #[must_use]
    pub fn on_leave(mut self, handler: impl Fn(&str, &str) + Send + Sync + 'static) -> Self {
        self.on_leave = Some(Arc::new(handler));
        self
    }

    /// A handler that sees every key BEFORE the grid (true = it used the key, the grid leaves it alone): the content of the cells and the
    /// grid then share ONE tab stop (the grid), as in a grid that holds its cells.
    #[must_use]
    pub fn first_key(
        mut self,
        handler: impl Fn(&KeyEvent) -> Handled + Send + Sync + 'static,
    ) -> Self {
        self.first_key = Some(Arc::new(handler));
        self
    }

    /// The id of the cell of a row and a column (`<id>-cell-<row>-<column>`; a spanning cell has the id of the first column).
    pub fn cell_id(&self, row: &str, col: &str) -> String {
        format!("{}-cell-{row}-{col}", self.id)
    }

    /// Whether the keys are inside a cell.
    pub fn inside(&self) -> bool {
        self.slot().inside()
    }

    /// The keys come back out of the cell: what the content calls when it gets Escape (it has the keyboard, not the grid).
    pub fn leave_cell(&self) {
        let told = {
            let mut state = self.slot();
            let at = Self::cursor_of(&state);
            state.leave().then_some(at).flatten()
        };
        self.live.refresh_now();
        if let (Some((row, col)), Some(handler)) = (told, &self.on_leave) {
            handler(&row, &col);
        }
    }

    fn cursor_of(state: &State) -> Option<(String, String)> {
        let (row, col) = state.cursor();
        let at = state.shown().get(row)?.node;
        Some((state.key_of(at).to_owned(), state.col_key(col).to_owned()))
    }

    pub(super) fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// How many columns the grid has.
    pub fn column_count(&self) -> usize {
        self.slot().cols().len()
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

    /// Put the cursor on the cell with these keys (a row and a column); false when it is not there (or not shown).
    pub fn go_to(&self, row: &str, col: &str) -> bool {
        let mut state = self.slot();
        let Some(node) = state.find(row) else {
            return false;
        };
        let at = (0..state.cols().len()).find(|&i| state.col_key(i) == col);
        at.is_some_and(|c| state.go(node, c))
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
                Outcome::Activated(n, c) | Outcome::Entered(n, c) | Outcome::Left(n, c) => {
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
        match outcome {
            Outcome::Entered(..) => Self::tell(self.on_enter.as_ref(), told),
            Outcome::Left(..) => Self::tell(self.on_leave.as_ref(), told),
            _ => self.said(told),
        }
        true
    }

    /// A click on the cell with these keys: the cursor goes there (the keys leave a cell they were in) and it is activated.
    pub(super) fn click(&self, row: &str, col: &str) {
        let told = {
            let mut state = self.slot();
            let at = (0..state.cols().len()).find(|&i| state.col_key(i) == col);
            let node = state.find(row);
            node.zip(at)
                .is_some_and(|(n, c)| state.go(n, c))
                .then(|| (row.to_owned(), col.to_owned()))
        };
        self.live.refresh_now();
        self.said(told);
    }

    fn said(&self, told: Option<(String, String)>) {
        Self::tell(self.on_activate.as_ref(), told);
    }

    fn tell(handler: Option<&OnActivate>, told: Option<(String, String)>) {
        if let (Some((row, col)), Some(handler)) = (told, handler) {
            handler(&row, &col);
        }
    }

    /// The grid, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let first = self.first_key.clone();
        let tree = self.slot().is_tree();
        let mut root = div()
            .id(&self.id)
            .class("fs-grid")
            .attr("role", if tree { "treegrid" } else { "grid" });
        if tree {
            root = root.class("fs-treegrid");
        }
        if self.slot().holds() {
            root = root.class("fs-grid-cells-hold");
        }
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("rows", move || inner.rows())),
            Arc::new(move |event: KeyEvent| {
                first.as_ref().is_some_and(|f| f(&event)) || keys.press(event)
            }),
        )
    }
}
