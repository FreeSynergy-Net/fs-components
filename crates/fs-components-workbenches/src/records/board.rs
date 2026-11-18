//! `RecordsBoard`: the records of a program behind the board workbench, read only (K4).
//!
//! Every decision (hidden, locked, which commands, what to say) is the plan's, through the judge.
//!
//! Pattern: Adapter (`BoardSource`, `BoardEvents`, `DetailContent` over a [`Worker`]'s snapshot, as `MemoryBoard` over its
//! memory). Reading only: moving and adding answer with a reason ("not possible here yet"), never with silence; the write side
//! follows with `RecordsWrite`. Columns are the groups, cards the rows of a group in page order, `locked` the lock of the record.

use std::sync::{Arc, Mutex, PoisonError};

use super::drive::Worker;
use super::judge::Judge;
use super::map::{self, Note};
use super::shape::RecordsBinding;
use super::snapshot::Problem;
use super::snapshot::Status;
use crate::board::{
    Answer, BoardEvents, BoardSource, Column, ContextTarget, DetailContent, DetailFrame,
    FilterState, MoveRequest, NewCard, Reason, Supports,
};

/// The most commands of the program a context menu offers.
const MENU_MAX: usize = 8;

#[derive(Default)]
struct Ui {
    opened: Option<String>,
    filter: FilterState,
}

/// The board over records. Cheap to clone; clones share the worker and the reader's state.
#[derive(Clone)]
pub struct RecordsBoard {
    worker: Arc<Worker>,
    binding: Arc<RecordsBinding>,
    judge: Arc<dyn Judge>,
    own: Arc<Vec<Problem>>,
    ui: Arc<Mutex<Ui>>,
}

impl RecordsBoard {
    /// A board over a worker. The judge is the plan's rules (`fs-forms-bind`), handed in by the host; the worker must have been
    /// started with the same judge and binding.
    pub fn new(worker: Arc<Worker>, binding: RecordsBinding, judge: Arc<dyn Judge>) -> Self {
        Self {
            worker,
            own: Arc::new(map::binding_problems(&binding)),
            binding: Arc::new(binding),
            judge,
            ui: Arc::default(),
        }
    }

    /// What is wrong with the data right now: a source that does not answer, a refused page, a cut-off list. Empty = nothing.
    pub fn notes(&self) -> Vec<Note> {
        let status = self.worker.status();
        self.worker.read(|s| map::notes(s, &status, &self.own))
    }

    /// How the worker stands.
    pub fn status(&self) -> Status {
        self.worker.status()
    }

    /// The card the reader opened last (click or Enter), if any.
    pub fn opened_card(&self) -> Option<String> {
        self.ui().opened.clone()
    }

    /// The filter the reader asked for (the query side applies it; the adapter only keeps what was said).
    pub fn filter(&self) -> FilterState {
        self.ui().filter.clone()
    }

    fn ui(&self) -> std::sync::MutexGuard<'_, Ui> {
        self.ui.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn write_later(&self) -> Reason {
        let r = self.judge.write_later();
        Reason::new(r.code, r.text)
    }
}

impl BoardSource for RecordsBoard {
    fn columns(&self) -> Vec<Column> {
        self.worker.read(|snap| {
            snap.groups
                .iter()
                .map(|g| {
                    let mut c = Column::new(&g.key, g.title.as_deref().unwrap_or(&g.key));
                    c.limit = g.hint_limit.and_then(|l| usize::try_from(l).ok());
                    c.cards = snap
                        .in_group(&g.key)
                        .map(|r| map::card(r, &self.binding, &*self.judge))
                        .collect();
                    c
                })
                .collect()
        })
    }

    fn can_drop(&self, _card: &str, _column: &str) -> Result<(), Reason> {
        Err(self.write_later())
    }

    fn locked(&self, card: &str) -> Option<Reason> {
        self.worker
            .read(|s| s.row(card).and_then(|r| map::lock_reason(r, &*self.judge)))
    }

    fn supports(&self) -> Supports {
        Supports {
            reorder: true,
            quick_add: true,
            columns_edit: false,
            context: true,
        }
    }

    fn generation(&self) -> u64 {
        self.worker.version()
    }
}

impl BoardEvents for RecordsBoard {
    fn moved(&self, _request: &MoveRequest) -> Answer {
        Answer::Refused(self.write_later())
    }

    fn opened(&self, card: &str) {
        self.ui().opened = Some(card.to_owned());
    }

    fn new_card(&self, _card: &NewCard) -> Answer {
        Answer::Refused(self.write_later())
    }

    fn context(&self, target: &ContextTarget, _at: (f32, f32)) -> Vec<String> {
        let ContextTarget::Card(id) = target else {
            return Vec::new();
        };
        // what the plan offers for this record (rights first, then "coming"); this adapter adds only the open command
        self.worker.read(|s| {
            let mut ids = vec!["board.open".to_owned()];
            if let Some(row) = s.row(id) {
                ids.extend(self.judge.commands(row).into_iter().take(MENU_MAX));
            }
            ids
        })
    }

    fn filter_changed(&self, filter: &FilterState) {
        self.ui().filter = filter.clone();
    }
}

impl DetailContent for RecordsBoard {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        self.worker.read(|s| {
            let row = s.row(card)?;
            let path = row
                .group
                .as_deref()
                .and_then(|k| s.groups.iter().find(|g| g.key == k))
                .map(|g| g.title.clone().unwrap_or_else(|| g.key.clone()))
                .unwrap_or_default();
            Some(map::frame(row, &path, &self.binding, &*self.judge))
        })
    }
}
