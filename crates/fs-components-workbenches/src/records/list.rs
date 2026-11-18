//! `RecordsRows`: the records of a program behind the list widget, read only (K8).
//!
//! Pattern: Adapter (`RowSource` over a [`Worker`]'s snapshot). One row per record in page order, one text per slot in the role
//! "cell"; a hidden field shows its reason, a locked record carries its sentence as the mark. A source that never answered says
//! so as the reason the list shows in place of rows.

use std::sync::Arc;

use super::drive::Worker;
use super::judge::Judge;
use super::map;
use super::shape::RecordsBinding;
use super::snapshot::Status;
use fs_components_widgets::list_view::{Row, RowSource};

/// The rows over records.
#[derive(Clone)]
pub struct RecordsRows {
    worker: Arc<Worker>,
    binding: Arc<RecordsBinding>,
    judge: Arc<dyn Judge>,
}

impl RecordsRows {
    /// Rows over a worker, with the plan's rules.
    pub fn new(worker: Arc<Worker>, binding: RecordsBinding, judge: Arc<dyn Judge>) -> Self {
        Self {
            worker,
            binding: Arc::new(binding),
            judge,
        }
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
                let r = self.judge.stale(&f.code);
                return Err(format!("{} ({})", r.text, r.code));
            }
        }
        Ok(self.worker.read(|s| {
            s.rows
                .iter()
                .skip(first)
                .take(limit)
                .map(|r| {
                    let row = Row::new(r.id.clone(), map::cells(r, &self.binding, &*self.judge));
                    match map::lock_reason(r, &*self.judge) {
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
