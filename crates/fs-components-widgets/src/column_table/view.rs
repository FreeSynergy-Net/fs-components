//! The table as elements and the handle a program holds.
//!
//! Pattern: Facade + Observer (one clonable handle over [`Rows`], draws again after each edit) + Adapter (Blinc). Text cells are the
//! entry line of the plan widgets (Enter commits); the status cell steps through the program's choices on a press; the row menu is
//! three small buttons (up, down, remove). Saving calls the program's callback only when the check allows it, and when it does
//! not, the reason stands next to the button.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{ColumnCheck, ColumnRow, Rows};
use crate::live::Live;
use crate::plan::Entry;

/// The words of the table (the program's, in the reader's language).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableLabels {
    /// Heading cells: name, status, stage, limit, cards now.
    pub heads: [String; 5],
    /// "+ Add column".
    pub add: String,
    /// "Save".
    pub save: String,
    /// Names of the row buttons: up, down, remove.
    pub row_buttons: [String; 3],
    /// The name given to a new column.
    pub new_name: String,
}

impl Default for TableLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_owned();
        Self {
            heads: [s("Name"), s("Status"), s("Stage"), s("Limit"), s("Cards")],
            add: s("Add column"),
            save: s("Save"),
            row_buttons: [s("Up"), s("Down"), s("Remove")],
            new_name: s("New column"),
        }
    }
}

type OnSave = Arc<dyn Fn(&[ColumnRow]) + Send + Sync>;

pub(super) struct Inner {
    pub(super) rows: Rows,
    pub(super) on_save: Option<OnSave>,
}

/// The table of columns. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct ColumnTable {
    pub(super) shared: Arc<Mutex<Inner>>,
    pub(super) check: Arc<dyn ColumnCheck>,
    pub(super) statuses: Vec<(String, String)>,
    pub(super) labels: TableLabels,
    pub(super) live: Live,
    pub(super) id: String,
    pub(super) entries: Arc<Mutex<Vec<[Entry; 3]>>>,
}

impl ColumnTable {
    /// A table of `rows`, checked by `check`; `statuses` are the program's choices for the status cell (key, word).
    pub fn new(
        rows: Vec<ColumnRow>,
        check: Arc<dyn ColumnCheck>,
        statuses: Vec<(String, String)>,
    ) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                rows: Rows::new(rows),
                on_save: None,
            })),
            check,
            statuses,
            labels: TableLabels::default(),
            live: Live::new(),
            id: "fs-columns".into(),
            entries: Arc::default(),
        }
    }

    /// The element id prefix (rows are `<id>-row-<n>`, the save button `<id>-save`, the reason `<id>-reason`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: TableLabels) -> Self {
        self.labels = labels;
        self
    }

    /// What happens on Save (only when the check allows it): the program writes the columns.
    #[must_use]
    pub fn on_save(self, f: OnSave) -> Self {
        self.lock().on_save = Some(f);
        self
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The rows as they are now.
    pub fn rows(&self) -> Vec<ColumnRow> {
        self.lock().rows.rows().to_vec()
    }

    /// Run `edit` on the table and draw again.
    pub fn edit<T>(&self, edit: impl FnOnce(&mut Rows) -> T) -> T {
        let out = edit(&mut self.lock().rows);
        self.live.refresh();
        out
    }

    /// Press Save: the program's callback runs if the check allows it; the answer says whether it did.
    pub fn save(&self) -> bool {
        let (rows, on_save) = {
            let inner = self.lock();
            if !inner.rows.save(self.check.as_ref()).allowed {
                drop(inner);
                self.live.refresh();
                return false;
            }
            (inner.rows.rows().to_vec(), inner.on_save.clone())
        };
        if let Some(f) = on_save {
            f(&rows);
        }
        true
    }

    /// What Save would say now.
    pub fn save_state(&self) -> super::model::Save {
        self.lock().rows.save(self.check.as_ref())
    }

    /// The next status after `current` in the program's choices (wraps; the first if `current` is not among them).
    pub fn next_status(&self, current: &str) -> Option<String> {
        let at = self.statuses.iter().position(|(k, _)| k == current);
        let next = at.map_or(0, |i| (i + 1) % self.statuses.len().max(1));
        self.statuses.get(next).map(|(k, _)| k.clone())
    }

    fn entries_for(&self, rows: &[ColumnRow]) -> Vec<[Entry; 3]> {
        let mut held = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        while held.len() < rows.len() {
            held.push([Entry::new(""), Entry::new(""), Entry::new("")]);
        }
        held.truncate(rows.len());
        for (e, r) in held.iter().zip(rows) {
            // The cells show what the model holds; what is typed and not yet entered is replaced by the next redraw.
            if e[0].text() != r.name {
                e[0].set(&r.name);
            }
            if e[1].text() != r.stage {
                e[1].set(&r.stage);
            }
            let wip = r.wip.map(|w| w.to_string()).unwrap_or_default();
            if e[2].text() != wip {
                e[2].set(&wip);
            }
        }
        held.clone()
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl crate::Element + use<> {
        let this = self.clone();
        self.live.region("table", move || this.render())
    }

    fn render(&self) -> Div {
        let (rows, problems, save) = {
            let inner = self.lock();
            (
                inner.rows.rows().to_vec(),
                inner.rows.problems(self.check.as_ref()),
                inner.rows.save(self.check.as_ref()),
            )
        };
        let entries = self.entries_for(&rows);
        let mut table = div().class("fs-ct").flex_col();
        let mut head = div().class("fs-ct-head").flex_row();
        head = head.child(div().class("fs-ct-handle"));
        for word in &self.labels.heads {
            head = head.child(text(word.clone()).class("fs-ct-head-cell"));
        }
        table = table.child(head);
        for (n, row) in rows.iter().enumerate() {
            table = table.child(self.row(n, row, &entries[n], &problems));
        }
        let add = self.clone();
        let add_name = self.labels.new_name.clone();
        table = table.child(
            div()
                .id(format!("{}-add", self.id))
                .class("fs-ct-add")
                .cursor_pointer()
                .child(text(format!("+ {}", self.labels.add)))
                .on_click(move |_| {
                    add.edit(|r| r.add(&add_name));
                }),
        );
        let this = self.clone();
        let mut foot = div().class("fs-ct-foot").flex_row().items_center().child(
            div().id(format!("{}-save", self.id)).w_fit().child(
                cn::button(self.labels.save.clone())
                    .variant(ButtonVariant::Primary)
                    .disabled(!save.allowed)
                    .on_click(move |_| {
                        this.save();
                    }),
            ),
        );
        if let Some(reason) = save.reason {
            foot = foot.child(
                text(reason)
                    .class("fs-ct-reason")
                    .id(format!("{}-reason", self.id)),
            );
        }
        table.child(foot)
    }
}
