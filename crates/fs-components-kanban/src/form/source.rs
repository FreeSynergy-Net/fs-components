//! The board's data from the form: the rows of `cards-in-columns` as columns ([`columns_of`]), and what the host lets be done
//! (`can` of the write port) as what the board supports.
//!
//! Pattern: Ports and Adapters (the adapter side of the port `BoardSource`): the board is the core and owns the port; this
//! is the one source that knows channel names. It is an object adapter: it holds the rows and the write port and translates.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use fs_components_core::form_read::{FormInput, Heads};
use fs_components_core::form_write::FormWrite;
use fs_components_fields::Say;

use super::events::{COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, CREATE, MOVE};
use super::read::columns_of;
use crate::model::{Column, Operation};
use crate::swimlanes::{LaneHead, LanePlan};
use crate::{BoardSource, Reason};

/// The only grouping the board knows today: the cards stand in the lane of their `assignee` values.
const LANE_GROUPING: &str = "assignee";

/// The sentence for a refusal code of the host (`kanban.refused.<code>`), as a reason the board shows.
pub fn reason(say: &dyn Say, code: &str) -> Reason {
    Reason::new(code, say.say(&format!("kanban.refused.{code}")))
}

/// The columns of the form as a [`BoardSource`]; the rows are replaced by [`FormSource::set`] when the form is read again.
pub struct FormSource {
    input: Mutex<FormInput>,
    write: Arc<dyn FormWrite>,
    say: Arc<dyn Say>,
    generation: AtomicU64,
}

impl FormSource {
    /// A source over the rows `input`; `write` says what may be done, `say` words the reasons.
    pub fn new(input: FormInput, write: Arc<dyn FormWrite>, say: Arc<dyn Say>) -> Self {
        Self {
            input: Mutex::new(input),
            write,
            say,
            generation: AtomicU64::new(0),
        }
    }

    /// The form was read again: take the new rows (the generation counts up, so the board reloads).
    pub fn set(&self, input: FormInput) {
        *self.input.lock().unwrap_or_else(PoisonError::into_inner) = input;
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Reads the rows as they are now (the lock is held only while `f` runs).
    pub(super) fn read<R>(&self, f: impl FnOnce(&FormInput) -> R) -> R {
        f(&self.input.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The column with this key, as it is now.
    pub fn column(&self, key: &str) -> Option<Column> {
        self.columns().into_iter().find(|c| c.key == key)
    }
}

impl BoardSource for FormSource {
    fn columns(&self) -> Vec<Column> {
        columns_of(&self.input.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn locked(&self, card: &str) -> Option<Reason> {
        (!self.write.can(MOVE, Some(card))).then(|| reason(self.say.as_ref(), "locked"))
    }

    fn supports(&self, op: Operation) -> bool {
        let can = |channel| self.write.can(channel, None);
        match op {
            Operation::Move => can(MOVE),
            Operation::AddCard => can(CREATE),
            Operation::AddColumn => can(COLUMN_CREATE),
            Operation::Rename => can(COLUMN_RENAME),
            Operation::Limit => can(COLUMN_LIMIT),
        }
    }

    fn lane_plan(&self) -> LanePlan {
        match &self
            .input
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .heads
        {
            None => LanePlan::Off,
            Some(Heads::Missing) => LanePlan::Missing,
            Some(Heads::Rows(rows)) => LanePlan::On {
                grouping: LANE_GROUPING.into(),
                heads: rows
                    .iter()
                    .map(|r| {
                        LaneHead::new(&r.key, if r.label.is_empty() { &r.key } else { &r.label })
                    })
                    .collect(),
            },
        }
    }

    fn offers_menu(&self) -> bool {
        // the host has no menu to open yet
        false
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
}
