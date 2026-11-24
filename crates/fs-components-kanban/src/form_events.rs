//! The board on a form: the gestures of the board become calls of the channels `move` and `create` of the form
//! `cards-in-columns@1.2.0` (SW-2), through the write port (`FormWrite`).
//!
//! Pattern: Adapter (`BoardSource` and `BoardEvents` over a `FormWrite`). The host decides everything: a move is possible
//! when the channel `move` may be used (`can`), the target is the id of the column (the label is only what is shown), the place
//! is the neighbours: `after` and `before` are the record ids of the cards next to the drop place in the target column (none = at
//! that end, both none = an empty column). `order` is only read; the fractional key is made by the module, never here. A refusal of the host is handed on as its code.

use std::sync::Arc;

use fs_components_core::form_write::{FormWrite, WriteArg};

use crate::model::{Column, Operation};
use crate::source::{Answer, BoardEvents, BoardSource};

/// The channel that moves a card (`command`, optional in the form).
pub const MOVE: &str = "move";
/// The channel that makes a card (`command`, optional in the form).
pub const CREATE: &str = "create";

/// The gesture "a new column" (the host decides whether it may be used), carrying the name `name`.
pub const COLUMN_CREATE: &str = "column.create";
/// The gesture "rename a column": the record is the key of the column, carrying the new `name`.
pub const COLUMN_RENAME: &str = "column.rename";
/// The gesture "limit a column": the record is the key of the column, carrying the number `limit` (0 = no limit).
pub const COLUMN_LIMIT: &str = "column.limit";
/// The argument with the name of a column.
pub const NAME: &str = "name";
/// The argument with the limit of a column.
pub const LIMIT: &str = "limit";

/// The channel with the id of the target column (name of the channel in the form, not a field of a module).
pub const COLUMN: &str = "column";
/// The channel with the record id of the card before the place (the one above it; absent = at the start of the column).
pub const AFTER: &str = "after";
/// The channel with the record id of the card after the place (the one below it; absent = at the end of the column).
pub const BEFORE: &str = "before";
/// The channel with the title of a new card.
pub const TITLE: &str = "title";
/// The channel with the lane of the column a new card goes into (only sent when the column has one).
pub const LANE: &str = "lane";

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
    source: Arc<dyn BoardSource>,
}

impl FormEvents {
    /// Events that send through `write`; `source` gives the cards of the target column, from which the neighbours are taken.
    pub fn new(write: Arc<dyn FormWrite>, source: Arc<dyn BoardSource>) -> Self {
        Self { write, source }
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
        let cards = self.source.columns().into_iter().find(|c| c.key == to);
        let rest: Vec<&str> = cards
            .iter()
            .flat_map(|c| c.cards.iter())
            .map(|c| c.key.as_str())
            .filter(|k| *k != key)
            .collect();
        let (after, before) = neighbours(&rest, index);
        for (name, id) in [(AFTER, after), (BEFORE, before)] {
            if let Some(id) = id {
                args.push(WriteArg::text(name, id));
            }
        }
        answer(self.write.send(MOVE, Some(key), &args))
    }

    fn add_card(&self, column: &str, title: &str) -> Answer {
        if !self.write.can(CREATE, None) {
            return Answer::Refused("locked".into());
        }
        let mut args = vec![WriteArg::text(TITLE, title), WriteArg::text(COLUMN, column)];
        let lane = self
            .source
            .columns()
            .into_iter()
            .find(|c| c.key == column)
            .and_then(|c| c.lane);
        if let Some(lane) = lane {
            args.push(WriteArg::text(LANE, &lane));
        }
        answer(self.write.send(CREATE, None, &args))
    }
}

/// The cards next to the place `index` among `rest` (the ids of the target column without the moved card, in order): the one
/// above (`after`) and the one below (`before`). `None` = at the end. An index past the end counts as the end.
fn neighbours<'a>(rest: &[&'a str], index: Option<usize>) -> (Option<&'a str>, Option<&'a str>) {
    let i = index.map_or(rest.len(), |i| i.min(rest.len()));
    (i.checked_sub(1).map(|p| rest[p]), rest.get(i).copied())
}
