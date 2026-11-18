//! `RecordsRows`: the records of a program behind the list widget, read only (K8).
//!
//! Pattern: Adapter (`RowSource` over a [`Worker`]'s snapshot). One row per record in page order, one text per slot in the role
//! "cell"; a hidden field shows its reason, a locked record carries its sentence as the mark. A source that never answered says
//! so as the reason the list shows in place of rows.

use std::sync::Arc;

use super::drive::Worker;
use super::map;
use super::present::{Present, join_values};
use super::shape::RecordsBinding;
use super::snapshot::Status;
use super::texts::RecordsTexts;
use fs_components_widgets::list_view::{Row, RowSource};

/// The rows over records.
#[derive(Clone)]
pub struct RecordsRows {
    worker: Arc<Worker>,
    binding: Arc<RecordsBinding>,
    texts: Arc<RecordsTexts>,
    present: Present,
}

impl RecordsRows {
    /// Rows over a worker.
    pub fn new(worker: Arc<Worker>, binding: RecordsBinding, texts: RecordsTexts) -> Self {
        Self {
            worker,
            binding: Arc::new(binding),
            texts: Arc::new(texts),
            present: Arc::new(join_values),
        }
    }

    /// With the way field values become words.
    #[must_use]
    pub fn presenting(mut self, present: Present) -> Self {
        self.present = present;
        self
    }
}

impl RowSource for RecordsRows {
    fn count(&self) -> usize {
        self.worker.read(|s| s.rows.len())
    }

    fn rows(&self, first: usize, limit: usize) -> Result<Vec<Row>, String> {
        if let Status::Stale(f) = self.worker.status() {
            // never answered: say so in place of the rows; answered before: the old rows stay and the board's notes say they are old
            if !self.worker.loaded() {
                return Err(format!("{} ({})", self.texts.stale, f.code));
            }
        }
        Ok(self.worker.read(|s| {
            s.rows
                .iter()
                .skip(first)
                .take(limit)
                .map(|r| {
                    let row = Row::new(
                        r.id.clone(),
                        map::cells(r, &self.binding, &self.present, &self.texts),
                    );
                    match map::lock_reason(r, &self.texts) {
                        Some(reason) => row.with_mark(reason.text),
                        None => row,
                    }
                })
                .collect()
        }))
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        self.worker
            .read(|s| s.rows.iter().position(|r| r.id == key))
    }
}
