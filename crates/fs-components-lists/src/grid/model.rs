//! The pure state of a grid or a treegrid.
//!
//! Pattern: State — the columns, the rows in an arena (indices, no references), the open marks, the cursor cell, where the keys are
//! (on the cursor or inside a cell), what a key does; no element, no callback.

use blinc_theme::SpacingToken;

use super::cell::Cell;
use super::hold::{Hold, hold};
use crate::keys::{Key, KeyEvent};

/// One column.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// What the caller gets back.
    pub key: String,
    /// The head cell's words.
    pub label: String,
}

impl Column {
    /// A column.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// One row as the caller gives it: a key, its cells (one per column) and, in a treegrid, child rows.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// What the caller gets back; part of the id (`<id>-row-<key>`).
    pub key: String,
    /// The text of each cell.
    pub cells: Vec<String>,
    /// Child rows (a treegrid shows them under an open parent; a grid ignores them).
    pub children: Vec<Row>,
    /// A parent that starts open.
    pub open: bool,
    /// The row has ONE cell that spans every column ([`Row::spanning`]).
    pub spans: bool,
    /// The row is at least this high (a theme spacing token, never a number of ours); `None` = as high as its content.
    pub min_height: Option<SpacingToken>,
}

impl Row {
    /// A row.
    pub fn new(key: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            key: key.into(),
            cells,
            children: Vec::new(),
            open: false,
            spans: false,
            min_height: None,
        }
    }

    /// A row of ONE cell over every column (the head of a lane, a parent row's title); `label` is what it says.
    pub fn spanning(key: impl Into<String>, label: impl Into<String>) -> Self {
        let mut row = Self::new(key, vec![label.into()]);
        row.spans = true;
        row
    }

    /// The same row, at least this high.
    #[must_use]
    pub fn min_height(mut self, min: SpacingToken) -> Self {
        self.min_height = Some(min);
        self
    }

    /// The same row with these children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Row>) -> Self {
        self.children = children;
        self
    }

    /// The same row, open.
    #[must_use]
    pub fn opened(mut self) -> Self {
        self.open = true;
        self
    }
}

struct Node {
    key: String,
    cells: Vec<String>,
    parent: Option<usize>,
    children: Vec<usize>,
    open: bool,
    spans: bool,
    min: Option<SpacingToken>,
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the grid.
    Ignored,
    /// The cursor moved.
    Moved,
    /// A parent row opened (its node).
    Opened(usize),
    /// A parent row closed (its node).
    Closed(usize),
    /// The cell was activated: the row's node and the column.
    Activated(usize, usize),
    /// The keys went into the cell (variant `cells-hold`): the row's node and the column.
    Entered(usize, usize),
    /// The keys came back out of the cell: the row's node and the column.
    Left(usize, usize),
}

/// One visible row as the view draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    /// The node.
    pub node: usize,
    /// Depth, 1 for a top row.
    pub level: usize,
    /// Siblings, itself included.
    pub set_size: usize,
    /// Its place among them, 1-based.
    pub pos: usize,
    /// `Some(open)` for a parent (treegrid only).
    pub expanded: Option<bool>,
}

/// The columns, the rows and the cursor.
pub struct State {
    cols: Vec<Column>,
    nodes: Vec<Node>,
    roots: Vec<usize>,
    tree: bool,
    page: usize,
    cursor: (usize, usize),
    /// The column an up or down move aims for (a row with fewer cells takes the nearest one; the aim stays).
    want: usize,
    holds: bool,
    inside: bool,
}

impl State {
    /// A grid (`tree` false: child rows are ignored) or a treegrid of these rows; the cursor starts on the first cell.
    pub fn new(cols: Vec<Column>, rows: &[Row], tree: bool) -> Self {
        let mut s = Self {
            cols,
            nodes: Vec::new(),
            roots: Vec::new(),
            tree,
            page: 10,
            cursor: (0, 0),
            want: 0,
            holds: false,
            inside: false,
        };
        s.roots = rows.iter().map(|r| s.add(r, None)).collect();
        s
    }

    fn add(&mut self, row: &Row, parent: Option<usize>) -> usize {
        let at = self.nodes.len();
        // a plain grid has no child rows, so no row of it is ever a parent and `expanded` stays `None`
        let kids: &[Row] = if self.tree { &row.children } else { &[] };
        self.nodes.push(Node {
            key: row.key.clone(),
            cells: row.cells.clone(),
            parent,
            children: Vec::new(),
            open: row.open && !kids.is_empty(),
            spans: row.spans,
            min: row.min_height,
        });
        let children: Vec<usize> = kids.iter().map(|c| self.add(c, Some(at))).collect();
        self.nodes[at].children = children;
        at
    }

    /// How many rows a Page Up or Page Down moves (at least one).
    pub fn set_page(&mut self, page: usize) {
        self.page = page.max(1);
    }

    /// Whether the cells hold elements and take the keys (variant `cells-hold`).
    pub fn set_holds(&mut self, holds: bool) {
        self.holds = holds;
        self.inside &= holds;
    }

    /// Whether the cells hold elements.
    pub fn holds(&self) -> bool {
        self.holds
    }

    /// Whether the keys are inside the cell under the cursor.
    pub fn inside(&self) -> bool {
        self.inside
    }

    /// The keys come back out of the cell from the program (the content got Escape); false when they were not inside.
    pub fn leave(&mut self) -> bool {
        std::mem::take(&mut self.inside)
    }

    /// The cell of a node in a column (a spanning row has the one cell in column 0).
    pub fn cell(&self, node: usize, col: usize) -> Cell {
        let n = &self.nodes[node];
        let words = n.cells.get(col).cloned().unwrap_or_default();
        if n.spans {
            Cell::Spans(words)
        } else {
            Cell::Text(words)
        }
    }

    /// The least height of a row (a spacing token of the theme).
    pub fn min_of(&self, node: usize) -> Option<SpacingToken> {
        self.nodes[node].min
    }

    /// The last column a row has: 0 for a spanning row.
    fn last_col_of(&self, node: usize) -> usize {
        if self.nodes[node].spans {
            0
        } else {
            self.cols.len().saturating_sub(1)
        }
    }

    /// The columns.
    pub fn cols(&self) -> &[Column] {
        &self.cols
    }

    /// Whether the rows nest.
    pub fn is_tree(&self) -> bool {
        self.tree
    }

    /// The visible rows in order.
    pub fn shown(&self) -> Vec<Shown> {
        let mut out = Vec::new();
        self.walk(&self.roots, 1, &mut out);
        out
    }

    fn walk(&self, siblings: &[usize], level: usize, out: &mut Vec<Shown>) {
        for (p, &i) in siblings.iter().enumerate() {
            let n = &self.nodes[i];
            out.push(Shown {
                node: i,
                level,
                set_size: siblings.len(),
                pos: p + 1,
                expanded: (!n.children.is_empty()).then_some(n.open),
            });
            if n.open {
                self.walk(&n.children, level + 1, out);
            }
        }
    }

    /// The key of a node.
    pub fn key_of(&self, node: usize) -> &str {
        &self.nodes[node].key
    }

    /// The cells of a node (a row with fewer cells than columns shows the rest empty).
    pub fn cells_of(&self, node: usize) -> &[String] {
        &self.nodes[node].cells
    }

    /// The key of a column.
    pub fn col_key(&self, col: usize) -> &str {
        &self.cols[col].key
    }

    /// The cursor: the index among the VISIBLE rows and the column.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Put the cursor on this cell (the pointer's way); false when it is not there.
    pub fn go(&mut self, node: usize, col: usize) -> bool {
        let at = self.shown().iter().position(|s| s.node == node);
        match at {
            Some(row) if col <= self.last_col_of(node) => {
                self.set_cursor((row, col));
                self.want = col;
                true
            }
            _ => false,
        }
    }

    /// Move the cursor; the keys leave the cell they were in when the cursor leaves it.
    fn set_cursor(&mut self, to: (usize, usize)) {
        if to != self.cursor {
            self.inside = false;
        }
        self.cursor = to;
    }

    /// The node with this key.
    pub fn find(&self, key: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.key == key)
    }

    /// Whether a node is open.
    pub fn is_open(&self, node: usize) -> bool {
        self.nodes[node].open
    }

    /// Open or close a parent row from the program; false when nothing changed.
    pub fn set_open(&mut self, node: usize, open: bool) -> bool {
        if self.nodes[node].children.is_empty() || self.nodes[node].open == open {
            return false;
        }
        let keep = self.shown().get(self.cursor.0).map(|s| s.node);
        self.nodes[node].open = open;
        // the cursor stays on its row when that is still shown, else it goes to the nearest shown ancestor
        let rows = self.shown();
        let mut want = keep;
        while let Some(n) = want {
            if let Some(at) = rows.iter().position(|s| s.node == n) {
                self.set_cursor((at, self.cursor.1));
                return true;
            }
            want = self.nodes[n].parent;
        }
        self.set_cursor((0, self.cursor.1));
        true
    }

    /// What a key does.
    pub fn press(&mut self, event: KeyEvent) -> Outcome {
        let rows = self.shown();
        let (row, col) = self.cursor;
        let (Some(last_row), Some(last_col)) =
            (rows.len().checked_sub(1), self.cols.len().checked_sub(1))
        else {
            return Outcome::Ignored;
        };
        let node = rows[row.min(last_row)].node;
        match hold(self.holds, self.inside, self.nodes[node].spans, event.key) {
            Hold::Normal => {}
            Hold::Enter => {
                self.inside = true;
                return Outcome::Entered(node, col);
            }
            Hold::Leave => {
                self.inside = false;
                return Outcome::Left(node, col);
            }
            Hold::Content => return Outcome::Ignored,
        }
        // along a row the aim is the new column; up and down keep the aim (a spanning row only has column 0)
        let across = |me: &mut Self, r: usize, c: usize| {
            me.want = c;
            if (r, c) == me.cursor {
                return Outcome::Ignored;
            }
            me.set_cursor((r, c));
            Outcome::Moved
        };
        let down_to = |me: &mut Self, r: usize| {
            let c = me.want.min(me.last_col_of(rows[r].node));
            if (r, c) == me.cursor {
                return Outcome::Ignored;
            }
            me.set_cursor((r, c));
            Outcome::Moved
        };
        let last_col = last_col.min(self.last_col_of(node));
        match event.key {
            Key::Right if col == 0 && rows[row].expanded == Some(false) => {
                self.nodes[node].open = true;
                Outcome::Opened(node)
            }
            Key::Left if col == 0 && rows[row].expanded == Some(true) => {
                self.nodes[node].open = false;
                Outcome::Closed(node)
            }
            Key::Right if col < last_col => across(self, row, col + 1),
            Key::Left if col > 0 => across(self, row, col - 1),
            Key::Down if row < last_row => down_to(self, row + 1),
            Key::Up if row > 0 => down_to(self, row - 1),
            Key::Home if event.ctrl => across(self, 0, 0),
            Key::End if event.ctrl => {
                let c = self.last_col_of(rows[last_row].node);
                across(self, last_row, c)
            }
            Key::Home => across(self, row, 0),
            Key::End => across(self, row, last_col),
            Key::PageDown => down_to(self, (row + self.page).min(last_row)),
            Key::PageUp => down_to(self, row.saturating_sub(self.page)),
            Key::Enter | Key::Space | Key::Activate => Outcome::Activated(node, col),
            _ => Outcome::Ignored,
        }
    }
}
