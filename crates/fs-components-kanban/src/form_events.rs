//! The board on a form: the gestures of the board become calls of the channels `move` and `create` of the form
//! `cards-in-columns@1.1.0` (SW-2), through the write port (`FormWrite`).
//!
//! Pattern: Adapter (`BoardSource` and `BoardEvents` over a `FormWrite`). The host decides everything: a move is possible
//! when the channel `move` may be used (`can`), the target is the id of the column (the label is only what is shown), the place
//! is the channel `order` (counted without the card itself, absent = at the end). A refusal of the host is handed on as its code.

use std::sync::Arc;

use fs_components_core::form_write::{FormWrite, WriteArg};

use crate::model::{Column, Operation};
use crate::source::{Answer, BoardEvents, BoardSource};

/// The channel that moves a card (`command`, optional in the form).
pub const MOVE: &str = "move";
/// The channel that makes a card (`command`, optional in the form).
pub const CREATE: &str = "create";

/// The channel with the id of the target column (name of the channel in the form, not a field of a module).
pub const COLUMN: &str = "column";
/// The channel with the place in the target column (counted without the card itself; absent = at the end).
pub const ORDER: &str = "order";
/// The channel with the title of a new card.
pub const TITLE: &str = "title";

/// The columns of `inner`, with `Move` and `AddCard` supported exactly when the host lets the channels `move` and `create` be used.
pub struct FormSupports {
    inner: Arc<dyn BoardSource>,
    write: Arc<dyn FormWrite>,
}

impl FormSupports {
    /// The source over `inner` that asks `write` what may be done.
    pub fn new(inner: Arc<dyn BoardSource>, write: Arc<dyn FormWrite>) -> Self {
        Self { inner, write }
    }
}

impl BoardSource for FormSupports {
    fn columns(&self) -> Vec<Column> {
        self.inner.columns()
    }

    fn supports(&self, op: Operation) -> bool {
        match op {
            Operation::Move => self.write.can(MOVE, None),
            Operation::AddCard => self.write.can(CREATE, None),
            _ => self.inner.supports(op),
        }
    }
}

/// The gestures of the board as calls of `write`.
pub struct FormEvents {
    write: Arc<dyn FormWrite>,
}

impl FormEvents {
    /// Events that send through `write`.
    pub fn new(write: Arc<dyn FormWrite>) -> Self {
        Self { write }
    }
}

fn answer(
    r: Result<
        fs_components_core::form_write::WriteDone,
        fs_components_core::form_write::WriteRefused,
    >,
) -> Answer {
    match r {
        Ok(_) => Answer::Accepted,
        Err(refused) => Answer::Refused(refused.code),
    }
}

impl BoardEvents for FormEvents {
    fn move_card_at(&self, key: &str, _from: &str, to: &str, index: Option<usize>) -> Answer {
        if !self.write.can(MOVE, Some(key)) {
            return Answer::Refused("locked".into());
        }
        let mut args = vec![WriteArg::text(COLUMN, to)];
        if let Some(i) = index {
            args.push(WriteArg::number(ORDER, i as f64));
        }
        answer(self.write.send(MOVE, Some(key), &args))
    }

    fn add_card(&self, column: &str, title: &str) -> Answer {
        if !self.write.can(CREATE, None) {
            return Answer::Refused("locked".into());
        }
        answer(self.write.send(
            CREATE,
            None,
            &[WriteArg::text(TITLE, title), WriteArg::text(COLUMN, column)],
        ))
    }
}
