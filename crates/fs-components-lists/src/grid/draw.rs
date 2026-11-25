//! The rows of a grid as elements.
//!
//! Pattern: Builder over a snapshot – the state is read once under its lock and copied into plain values; the elements (and the
//! [`CellView`] of the caller, which may ask the grid anything) are made after the lock is gone.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_theme::ThemeState;

use super::cell::CellAt;
use super::model::{Column, State};
use super::view::Grid;

/// How far a level of a treegrid is indented, in pixels.
const INDENT: f32 = 16.0;

/// One visible row, copied out of the state.
struct RowSnap {
    key: String,
    level: usize,
    set_size: usize,
    pos: usize,
    expanded: Option<bool>,
    spans: bool,
    min: Option<blinc_theme::SpacingToken>,
    cells: Vec<String>,
}

/// What the elements are made from.
struct Snap {
    cols: Vec<Column>,
    rows: Vec<RowSnap>,
    cursor: (usize, usize),
    tree: bool,
    inside: bool,
}

impl Snap {
    fn of(state: &State) -> Self {
        let rows = state
            .shown()
            .iter()
            .map(|s| RowSnap {
                key: state.key_of(s.node).to_owned(),
                level: s.level,
                set_size: s.set_size,
                pos: s.pos,
                expanded: s.expanded,
                spans: state.cell(s.node, 0).spans(),
                min: state.min_of(s.node),
                cells: state.cells_of(s.node).to_vec(),
            })
            .collect();
        Self {
            cols: state.cols().to_vec(),
            rows,
            cursor: state.cursor(),
            tree: state.is_tree(),
            inside: state.inside(),
        }
    }
}

impl Grid {
    /// The table: the head row and the visible rows.
    pub(super) fn rows(&self) -> Div {
        let snap = Snap::of(&self.slot());
        let mut table = div()
            .id(format!("{}-table", self.id))
            .flex_col()
            .attr("aria-rowcount", (snap.rows.len() + 1).to_string())
            .attr("aria-colcount", snap.cols.len().to_string());
        let mut head = div()
            .id(format!("{}-head", self.id))
            .class("fs-grid-head")
            .attr("role", "row")
            .attr("aria-rowindex", "1");
        for (c, col) in snap.cols.iter().enumerate() {
            let mut cell = div()
                .id(format!("{}-headcell-{}", self.id, col.key))
                .class("fs-grid-headcell")
                .attr("role", "columnheader")
                .attr("aria-colindex", (c + 1).to_string())
                .w(self.width);
            cell = match self.view.head(&col.key) {
                Some(own) => cell.child(own),
                None => cell.child(text(col.label.clone()).class("fs-grid-headcell-text")),
            };
            head = head.child(cell);
        }
        table = table.child(head);
        for (r, row) in snap.rows.iter().enumerate() {
            table = table.child(self.row(&snap, row, r));
        }
        table
    }

    fn row(&self, snap: &Snap, row: &RowSnap, index: usize) -> Div {
        let mut line = div()
            .id(format!("{}-row-{}", self.id, row.key))
            .class("fs-grid-row")
            .attr("role", "row")
            .attr("aria-rowindex", (index + 2).to_string());
        if let Some(min) = row.min {
            line = line.min_h(ThemeState::get().spacing_value(min));
        }
        if snap.tree {
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
        let shown = if row.spans { 1 } else { snap.cols.len() };
        for c in 0..shown {
            line = line.child(self.cell(snap, row, index, c));
        }
        line
    }

    fn cell(&self, snap: &Snap, row: &RowSnap, index: usize, c: usize) -> Div {
        let col = &snap.cols[c].key;
        let key = &row.key;
        let current = snap.cursor == (index, c);
        let words = row.cells.get(c).cloned().unwrap_or_default();
        let mut cell = div()
            .id(self.cell_id(key, col))
            .class("fs-grid-cell")
            .attr("role", if row.spans { "rowheader" } else { "gridcell" })
            .attr("aria-colindex", (c + 1).to_string());
        if row.spans {
            #[allow(clippy::cast_precision_loss)]
            let all = snap.cols.len() as f32;
            cell = cell
                .class("fs-grid-rowhead")
                .attr("aria-colspan", snap.cols.len().to_string())
                .w(self.width * all);
        } else {
            cell = cell.w(self.width);
        }
        if current {
            cell = cell.class("fs-grid-cell-current");
            if snap.inside {
                cell = cell.class("fs-grid-cell-open");
            }
        }
        let at = CellAt::new(key, col, row.expanded == Some(true), current && snap.inside);
        let own = self.view.cell(&at);
        let (this, node_key, col_key) = (self.clone(), key.clone(), col.clone());
        let cell = if snap.tree && c == 0 {
            cell.child(Self::lead(row, own, words))
        } else {
            match own {
                Some(own) => cell.child(own),
                None => cell.child(text(words).class("fs-grid-cell-text")),
            }
        };
        cell.on_click(move |_| this.click(&node_key, &col_key))
    }

    /// The content of the first cell of a treegrid: behind the indent and the triangle of its level.
    fn lead(row: &RowSnap, own: Option<Div>, words: String) -> Div {
        let mark = match row.expanded {
            Some(true) => "\u{25BE} ",
            Some(false) => "\u{25B8} ",
            None => "",
        };
        #[allow(clippy::cast_precision_loss)]
        let indent = INDENT * (row.level - 1) as f32;
        let line = div()
            .flex_row()
            .child(div().w(indent).h(1.0).attr("aria-hidden", "true"))
            .child(
                div()
                    .class("fs-grid-mark")
                    .attr("aria-hidden", "true")
                    .child(text(mark)),
            );
        match own {
            Some(own) => line.child(own),
            None => line.child(text(words).class("fs-grid-cell-text")),
        }
    }
}
