//! The board on a form: the gestures of the board become the gestures `drop` (arguments `column`, `lane`, `after`, `before`: the slot
//! `card.drop`, the trigger word of the core) and `quick-add` of the slots of the block `board`
//! (contract 0.5.0), through the write port (`FormWrite::fire`); the host finds the `[[action]]` and maps it to the form channel.
//!
//! Pattern: Ports and Adapters (the adapter side of the port `BoardEvents`; the board is the core and owns the port). The
//! host decides everything: a move is possible when the gesture `drop` may be used (`may_fire`), the target is the id of the column (the
//! label is only what is shown), the place is the neighbours: `after` and `before` are the record ids of the cards next to the drop
//! place in the target column (none = at that end, both none = an empty column). `order` is only read; the fractional key is made
//! by the module, never here. A refusal of the host is handed on as its code (the reason carries the code and a sentence).

use std::sync::Arc;

use fs_components_core::form_write::trigger::{DROP, QUICK_ADD};
use fs_components_core::form_write::{FormWrite, Gesture, WriteArg, WriteDone, WriteRefused};
use fs_components_fields::Say;

use super::source::reason;
use crate::{Answer, BoardEvents, BoardSource, MoveRequest, NewCard};

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

/// The argument of the slot with the id of the target column (name of the channel in the form, not a field of a module).
pub const COLUMN: &str = "column";
/// The argument of the slot with the record id of the card before the place (the one above it; absent = at the start of the column).
pub const AFTER: &str = "after";
/// The argument of the slot with the record id of the card after the place (the one below it; absent = at the end of the column).
pub const BEFORE: &str = "before";
/// The argument of the slot with the title of a new card.
pub const TITLE: &str = "title";
/// The argument of the slot with the lane of the column a new card goes into (only sent when the column has one).
pub const LANE: &str = "lane";

/// The gestures of the board as calls of `write`: a move and a new card.
pub struct FormEvents {
    write: Arc<dyn FormWrite>,
    source: Arc<dyn BoardSource>,
    say: Arc<dyn Say>,
}

impl FormEvents {
    /// Events that send through `write`; `source` gives the cards of the target column (from which the neighbours are taken),
    /// `say` words a refusal of the host.
    pub fn new(write: Arc<dyn FormWrite>, source: Arc<dyn BoardSource>, say: Arc<dyn Say>) -> Self {
        Self { write, source, say }
    }

    fn answer(&self, r: Result<WriteDone, WriteRefused>) -> Answer {
        match r {
            Ok(_) => Answer::Accepted,
            Err(refused) => Answer::Refused(reason(self.say.as_ref(), &refused.code)),
        }
    }
}

impl BoardEvents for FormEvents {
    fn moved(&self, r: &MoveRequest) -> Answer {
        if let Err(refused) = self.write.may_fire(Gesture::new(DROP), Some(&r.card)) {
            return Answer::Refused(reason(self.say.as_ref(), &refused.code));
        }
        let mut args = vec![WriteArg::text(COLUMN, &r.to)];
        let cards = self.source.columns().into_iter().find(|c| c.key == r.to);
        let rest: Vec<&str> = cards
            .iter()
            .flat_map(|c| c.cards.iter())
            .map(|c| c.key.as_str())
            .filter(|k| *k != r.card)
            .collect();
        let (after, before) = neighbours(&rest, r);
        for (name, id) in [(AFTER, after), (BEFORE, before)] {
            if let Some(id) = id {
                args.push(WriteArg::text(name, id));
            }
        }
        // The lane the card was dropped in (slot `card.drop`, argument `lane`); a card dropped in the lane "without lane" sends none:
        // on a board with lanes that says "no value".
        if let Some(lane) = r.to_lane.as_deref().filter(|l| *l != crate::NONE_KEY) {
            args.push(WriteArg::text(LANE, lane));
        }
        self.answer(self.write.fire(Gesture::new(DROP), Some(&r.card), &args))
    }

    fn new_card(&self, card: &NewCard) -> Answer {
        if let Err(refused) = self.write.may_fire(Gesture::new(QUICK_ADD), None) {
            return Answer::Refused(reason(self.say.as_ref(), &refused.code));
        }
        let mut args = vec![
            WriteArg::text(TITLE, &card.title),
            WriteArg::text(COLUMN, &card.column),
        ];
        let lane = self
            .source
            .columns()
            .into_iter()
            .find(|c| c.key == card.column)
            .and_then(|c| c.lane);
        if let Some(lane) = lane {
            args.push(WriteArg::text(LANE, &lane));
        }
        self.answer(self.write.fire(Gesture::new(QUICK_ADD), None, &args))
    }
}

/// The cards next to the place of the request among `rest` (the ids of the target column without the moved card, in order): the
/// one above (`after`) and the one below (`before`). The place is the shown card it comes after (`after`, `None` = the top); a
/// card that is not there falls back to the number `index` (past the end counts as the end).
fn neighbours<'a>(rest: &[&'a str], r: &MoveRequest) -> (Option<&'a str>, Option<&'a str>) {
    let at = match r.after.as_deref() {
        None => 0,
        Some(a) => rest
            .iter()
            .position(|k| *k == a)
            .map_or(r.index.min(rest.len()), |p| p + 1),
    };
    (at.checked_sub(1).map(|p| rest[p]), rest.get(at).copied())
}
