//! The two boundaries to the program: where the board's data comes from, and who hears what the reader does.
//!
//! Pattern: Strategy (`BoardSource`, `BoardEvents`) + Adapter (the program puts its store behind them) + Null object (`NoEvents`).
//! The workbench knows no program and no store: a column key can be a status or a stage, a card key a task or a picture. It
//! writes nothing, ranks nothing: every change is a request that the program answers (`Answer`), and the board draws the
//! next `columns()`. The test double is `MemoryBoard` (`memory.rs`).

use crate::Column;

use crate::Operation;
use crate::requests::{
    Answer, ColumnAction, ContextTarget, FilterState, MoveRequest, NewCard, Reason,
};

/// Pattern: Adapter - the program's data behind one read port; the board never stores a card of its own.
///
/// Where the columns and cards come from.
pub trait BoardSource: Send + Sync {
    /// The columns with the cards that are **shown** (a filter has been applied), in order.
    fn columns(&self) -> Vec<Column>;

    /// How many cards of this column the filter hides ("3 hidden"); the work-in-progress mark counts them too.
    fn hidden(&self, _column: &str) -> usize {
        0
    }

    /// May `card` be put into `column`? The program decides (rights, stages, a claim); the board shows the reason and does not move.
    /// A work-in-progress limit is **not** a reason: it warns, it never locks.
    ///
    /// # Errors
    /// The [`Reason`] it may not.
    fn can_drop(&self, _card: &str, _column: &str) -> Result<(), Reason> {
        Ok(())
    }

    /// Why `card` cannot be picked up at all (it is locked), if it cannot.
    fn locked(&self, _card: &str) -> Option<Reason> {
        None
    }

    /// Does the program take this operation now? One question per [`Operation`]; what is not supported is drawn disabled (reason in a tooltip) with the
    /// word "soon" (ADR 0096 H-K5), never left out and never dead without a word. Read on every build: a control unlocks by itself.
    fn supports(&self, _op: Operation) -> bool {
        true
    }

    /// Can the table of columns be opened (any of add a column, rename, set the limit)?
    fn columns_editable(&self) -> bool {
        [Operation::AddColumn, Operation::Rename, Operation::Limit]
            .into_iter()
            .any(|op| self.supports(op))
    }

    /// Does the host open a context menu (a right click, the "..." button of a column)? Not a program operation: the host decides.
    fn offers_menu(&self) -> bool {
        true
    }

    /// A number that changes whenever the data changed (a write of anyone): the board reloads when it differs from the last one.
    fn generation(&self) -> u64 {
        0
    }
}

/// Pattern: Observer - the board tells the program what happened; every method has a default, so a program listens only to what it needs.
///
/// What the board tells the program. Every method has a default that does nothing (or refuses).
pub trait BoardEvents: Send + Sync {
    /// A card was moved.
    fn moved(&self, _request: &MoveRequest) -> Answer {
        Answer::Refused(Reason::new("unsupported", "Moving is not supported here."))
    }

    /// A card is to be opened (click, or Enter).
    fn opened(&self, _card: &str) {}

    /// A card is to be made from what was typed at the foot of a column.
    fn new_card(&self, _card: &NewCard) -> Answer {
        Answer::Refused(Reason::new("unsupported", "Adding is not supported here."))
    }

    /// The commands the context menu of `target` offers (ids of the command model, `board.*` and the program's own), at most 8 of the
    /// program's. The shell (`fs-components-app`) opens the menu; the workbench only asks for the lines.
    fn context(&self, _target: &ContextTarget, _at: (f32, f32)) -> Vec<String> {
        Vec::new()
    }

    /// The reader changed the filter.
    fn filter_changed(&self, _filter: &FilterState) {}

    /// The reader asked something of a column's short menu.
    fn column_action(&self, _column: &str, _action: ColumnAction) {}
}

/// Hears nothing; refuses every move and every new card.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl BoardEvents for NoEvents {}
