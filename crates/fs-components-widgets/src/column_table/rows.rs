//! One row of the table and its cells.
//!
//! Pattern: Adapter (split from `view.rs`).

use super::model::ColumnRow;
use super::view::ColumnTable;
use crate::plan::Entry;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use std::sync::Arc;
impl ColumnTable {
    #[allow(clippy::many_single_char_names)] // a, b, c: one handle per cell callback
    pub(super) fn row(
        &self,
        n: usize,
        row: &ColumnRow,
        cells: &[Entry; 3],
        problems: &[super::model::Problem],
    ) -> Div {
        let id = &self.id;
        let mut d = div()
            .id(format!("{id}-row-{n}"))
            .class("fs-ct-row")
            .flex_row()
            .items_center()
            .child(div().class("fs-ct-handle").child(text("\u{2830}")));
        let (name, stage, wip) = (cells[0].clone(), cells[1].clone(), cells[2].clone());
        let (a, b, c) = (self.clone(), self.clone(), self.clone());
        d = d
            .child(self.cell(
                id,
                n,
                "name",
                &name,
                Arc::new(move |t: &str, _| {
                    a.edit(|r| r.set_name(n, t));
                }),
            ))
            .child({
                let this = self.clone();
                let key = row.status.clone();
                let word = self
                    .statuses
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map_or_else(|| key.clone(), |(_, w)| w.clone());
                div()
                    .id(format!("{id}-status-{n}"))
                    .class("fs-ct-cell")
                    .class("fs-ct-status")
                    .cursor_pointer()
                    .child(text(word))
                    .on_click(move |_| {
                        if let Some(next) = this.next_status(&key) {
                            this.edit(|r| r.set_status(n, &next));
                        }
                    })
            })
            .child(self.cell(
                id,
                n,
                "stage",
                &stage,
                Arc::new(move |t: &str, _| {
                    b.edit(|r| r.set_stage(n, t));
                }),
            ))
            .child(self.cell(
                id,
                n,
                "wip",
                &wip,
                Arc::new(move |t: &str, _| {
                    c.edit(|r| r.set_wip(n, t));
                }),
            ))
            .child(text(row.cards.to_string()).class("fs-ct-cell"));
        for (i, word) in self.labels.row_buttons.iter().enumerate() {
            let this = self.clone();
            d = d.child(
                div()
                    .id(format!("{id}-{}-{n}", ["up", "down", "remove"][i]))
                    .class("fs-ct-rowbtn")
                    .attr("role", "button")
                    .attr("aria-label", word.as_str())
                    .cursor_pointer()
                    .child(text(["\u{25b2}", "\u{25bc}", "\u{2715}"][i]))
                    .on_click(move |_| match i {
                        0 => {
                            this.edit(|r| r.move_row(n, n.saturating_sub(1)));
                        }
                        1 => {
                            this.edit(|r| r.move_row(n, n + 1));
                        }
                        _ => {
                            // A row with cards needs its target: the previous row, or the next one for the first.
                            let into = if n == 0 { 1 } else { n - 1 };
                            let _ = this.edit(|r| r.remove(n, Some(into)));
                        }
                    }),
            );
        }
        for p in problems.iter().filter(|p| p.row == Some(n)) {
            d = d.child(text(p.text.clone()).class("fs-ct-problem"));
        }
        d
    }

    #[allow(clippy::unused_self)] // a method so the cells read as the table's own
    pub(super) fn cell(
        &self,
        id: &str,
        n: usize,
        tag: &str,
        entry: &Entry,
        on_enter: crate::plan::OnEnter,
    ) -> Div {
        div()
            .class("fs-ct-cell")
            .child(entry.build_with(&format!("{id}-{tag}-{n}"), on_enter))
    }
}
