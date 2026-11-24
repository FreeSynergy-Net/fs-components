//! The two boundaries to the program: where the board's data comes from, and who hears what the reader does.
//!
//! Pattern: Strategy (`BoardSource`, `BoardEvents`) + Adapter (the program puts its store behind them) + Null object (`NoEvents`).
//! The workbench knows no program and no store: a column key can be a status or a stage, a card key a task or a picture. It
//! writes nothing, ranks nothing: every change is a request that the program answers (`Answer`), and the board draws the
//! next `columns()`. The doubles for tests are at the end.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::{Card, Column};

use super::model::{
    Answer, ColumnAction, ContextTarget, FilterState, MoveRequest, NewCard, Reason, Supports,
};

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

    /// What is supported now.
    fn supports(&self) -> Supports {
        Supports::default()
    }

    /// A number that changes whenever the data changed (a write of anyone): the board reloads when it differs from the last one.
    fn generation(&self) -> u64 {
        0
    }
}

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

/// A board in memory for tests and demos: it keeps its columns, answers as it is told and counts the writes.
#[derive(Clone)]
pub struct MemoryBoard {
    inner: Arc<Mutex<Memory>>,
    generation: Arc<AtomicU64>,
}

struct Memory {
    columns: Vec<Column>,
    hidden: Vec<(String, usize)>,
    refuse_into: Vec<(String, Reason)>,
    locked: Vec<(String, Reason)>,
    ask_for: Vec<String>,
    refuse_moves: Option<Reason>,
    log: Vec<String>,
}

impl MemoryBoard {
    /// A board with these columns.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Memory {
                columns,
                hidden: Vec::new(),
                refuse_into: Vec::new(),
                locked: Vec::new(),
                ask_for: Vec::new(),
                refuse_moves: None,
                log: Vec::new(),
            })),
            generation: Arc::default(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Memory> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `n` cards of `column` are hidden by the filter.
    #[must_use]
    pub fn with_hidden(self, column: &str, n: usize) -> Self {
        self.lock().hidden.push((column.to_owned(), n));
        self
    }

    /// Nothing may be dropped into `column`.
    #[must_use]
    pub fn refusing(self, column: &str, reason: Reason) -> Self {
        self.lock().refuse_into.push((column.to_owned(), reason));
        self
    }

    /// `card` cannot be picked up.
    #[must_use]
    pub fn locking(self, card: &str, reason: Reason) -> Self {
        self.lock().locked.push((card.to_owned(), reason));
        self
    }

    /// A move of `card` is answered with a question until it is confirmed.
    #[must_use]
    pub fn asking_about(self, card: &str) -> Self {
        self.lock().ask_for.push(card.to_owned());
        self
    }

    /// Every move is refused with this reason (the store says no after the board has already drawn the card at its new place).
    #[must_use]
    pub fn refusing_moves(self, reason: Reason) -> Self {
        self.lock().refuse_moves = Some(reason);
        self
    }

    /// The columns as they are now.
    pub fn snapshot(&self) -> Vec<Column> {
        self.lock().columns.clone()
    }

    /// What the program was asked, one line per call (`moved:card:from>to@after`, `new:column:title:top`, `opened:card`).
    pub fn log(&self) -> Vec<String> {
        self.lock().log.clone()
    }

    /// Change the data as another writer would (the generation counts up).
    pub fn put(&self, columns: Vec<Column>) {
        self.lock().columns = columns;
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl BoardSource for MemoryBoard {
    fn columns(&self) -> Vec<Column> {
        self.snapshot()
    }
    fn hidden(&self, column: &str) -> usize {
        self.lock()
            .hidden
            .iter()
            .find(|(c, _)| c == column)
            .map_or(0, |(_, n)| *n)
    }
    fn can_drop(&self, _card: &str, column: &str) -> Result<(), Reason> {
        match self.lock().refuse_into.iter().find(|(c, _)| c == column) {
            Some((_, reason)) => Err(reason.clone()),
            None => Ok(()),
        }
    }
    fn locked(&self, card: &str) -> Option<Reason> {
        self.lock()
            .locked
            .iter()
            .find(|(c, _)| c == card)
            .map(|(_, r)| r.clone())
    }
    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
}

impl BoardEvents for MemoryBoard {
    fn moved(&self, r: &MoveRequest) -> Answer {
        let mut m = self.lock();
        m.log.push(format!(
            "moved:{}:{}>{}@{}{}",
            r.card,
            r.from,
            r.to,
            r.after.as_deref().unwrap_or("^"),
            if r.confirmed { ":yes" } else { "" }
        ));
        if let Some(reason) = m.refuse_moves.clone() {
            return Answer::Refused(reason);
        }
        if !r.confirmed && m.ask_for.contains(&r.card) {
            return Answer::Ask(super::model::Question::new(
                "Held by an agent - move anyway?",
            ));
        }
        let mut card = None;
        for c in &mut m.columns {
            if let Some(i) = c.cards.iter().position(|k| k.key == r.card) {
                card = Some(c.cards.remove(i));
            }
        }
        let Some(card) = card else {
            return Answer::Refused(Reason::new("gone", "The card is gone."));
        };
        let Some(to) = m.columns.iter_mut().find(|c| c.key == r.to) else {
            return Answer::Refused(Reason::new("no-column", "No such column."));
        };
        let at = r
            .after
            .as_deref()
            .and_then(|a| to.cards.iter().position(|k| k.key == a))
            .map_or(0, |i| i + 1);
        to.cards.insert(at, card);
        drop(m);
        self.generation.fetch_add(1, Ordering::Relaxed);
        Answer::Accepted
    }

    fn opened(&self, card: &str) {
        self.lock().log.push(format!("opened:{card}"));
    }

    fn new_card(&self, n: &NewCard) -> Answer {
        let mut m = self.lock();
        m.log
            .push(format!("new:{}:{}:{}", n.column, n.title, n.top));
        let Some(column) = m.columns.iter_mut().find(|c| c.key == n.column) else {
            return Answer::Refused(Reason::new("no-column", "No such column."));
        };
        let card = Card::new(&format!("n{}", column.cards.len()), &n.title);
        if n.top {
            column.cards.insert(0, card);
        } else {
            column.cards.push(card);
        }
        drop(m);
        self.generation.fetch_add(1, Ordering::Relaxed);
        Answer::Accepted
    }

    fn context(&self, target: &ContextTarget, _at: (f32, f32)) -> Vec<String> {
        match target {
            ContextTarget::Card(_) => {
                vec!["board.open".into(), "board.up".into(), "board.down".into()]
            }
            ContextTarget::Column(_) => vec!["board.rename".into()],
            ContextTarget::Board => vec!["board.new".into()],
        }
    }
}
