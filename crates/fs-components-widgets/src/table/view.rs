//! The table as an element.
//!
//! Pattern: Builder — described by chained calls, drawn by `build`; it holds no state after that.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Element;
use crate::grid::Column;

/// How wide a column is unless told otherwise, in pixels.
const COLUMN_WIDTH: f32 = 140.0;

/// A table of columns and rows of text.
#[must_use]
pub struct Table {
    id: String,
    caption: Option<String>,
    name: Option<String>,
    cols: Vec<Column>,
    rows: Vec<Vec<String>>,
    row_headers: bool,
    of: Option<(usize, usize)>,
    width: f32,
}

impl Table {
    /// A table of these columns and rows (one `Vec` of cells per row). Ids: `<id>`, `<id>-caption`, `<id>-row-<index>`.
    pub fn new(cols: Vec<Column>, rows: Vec<Vec<String>>) -> Self {
        Self {
            id: "table".to_owned(),
            caption: None,
            name: None,
            cols,
            rows,
            row_headers: false,
            of: None,
            width: COLUMN_WIDTH,
        }
    }

    /// The prefix of the ids.
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A visible caption; it also names the table (`aria-labelledby`).
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_owned());
        self
    }

    /// A name for a table without a caption (`aria-label`).
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// The first cell of every row names the row (`rowheader`).
    pub fn row_headers(mut self) -> Self {
        self.row_headers = true;
        self
    }

    /// The table shows a part of a larger one of `rows` rows and `cols` columns (`aria-rowcount`, `aria-colcount`).
    pub fn of(mut self, rows: usize, cols: usize) -> Self {
        self.of = Some((rows, cols));
        self
    }

    /// How wide a column is, in pixels.
    pub fn column_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// The table, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let mut root = div().id(&self.id).class("fs-table").attr("role", "table");
        let caption_id = format!("{}-caption", self.id);
        if self.caption.is_some() {
            root = root.attr("aria-labelledby", &caption_id);
        } else if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        if let Some((rows, cols)) = self.of {
            // the head is a row of its own
            root = root
                .attr("aria-rowcount", (rows + 1).to_string())
                .attr("aria-colcount", cols.to_string());
        }
        if let Some(caption) = &self.caption {
            root = root.child(div().id(caption_id).class("fs-table-caption").child(text(caption.clone())));
        }
        let mut head = div()
            .class("fs-table-head")
            .attr("role", "row");
        for (c, col) in self.cols.iter().enumerate() {
            head = head.child(self.cell(format!("{}-head-{c}", self.id), &col.label, "columnheader", "fs-table-headcell-text", c));
        }
        root = root.child(div().attr("role", "rowgroup").child(head));
        let mut body = div().attr("role", "rowgroup").flex_col();
        for (r, cells) in self.rows.iter().enumerate() {
            let mut row = div()
                .id(format!("{}-row-{r}", self.id))
                .class("fs-table-row")
                .attr("role", "row");
            for c in 0..self.cols.len() {
                let words = cells.get(c).map_or("", String::as_str);
                let id = format!("{}-cell-{r}-{c}", self.id);
                row = row.child(if c == 0 && self.row_headers {
                    self.cell(id, words, "rowheader", "fs-table-rowhead-text", c)
                } else {
                    self.cell(id, words, "cell", "fs-table-cell-text", c)
                });
            }
            body = body.child(row);
        }
        root.child(body)
    }

    fn cell(&self, id: String, words: &str, role: &str, text_class: &str, c: usize) -> Div {
        let mut cell = div()
            .id(id)
            .class("fs-table-cell")
            .attr("role", role)
            .w(self.width)
            .child(text(words.to_owned()).class(text_class.to_owned()));
        if self.of.is_some() {
            cell = cell.attr("aria-colindex", (c + 1).to_string());
        }
        cell
    }
}
