//! A board in memory for tests and demos: it keeps its columns, answers as it is told and counts the writes.
//!
//! Pattern: Test double (Fake): a [`BoardSource`] and [`BoardEvents`] over plain data. Split from `source.rs`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::{Card, Column, Operation};

use crate::requests::{Answer, ContextTarget, MoveRequest, NewCard, Reason};
use crate::source::{BoardEvents, BoardSource};
use crate::swimlanes::LanePlan;

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
    off: Vec<Operation>,
    no_menu: bool,
    lanes: LanePlan,
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
                off: Vec::new(),
                no_menu: false,
                lanes: LanePlan::Off,
                log: Vec::new(),
            })),
            generation: Arc::default(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Memory> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The board draws lanes by this plan.
    #[must_use]
    pub fn with_lanes(self, plan: LanePlan) -> Self {
        self.lock().lanes = plan;
        self
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

    /// `op` is not supported (the control is drawn disabled, the reason in a tooltip).
    #[must_use]
    pub fn without(self, op: Operation) -> Self {
        self.lock().off.push(op);
        self
    }

    /// `op` is supported again (a program that learned it: the locked control unlocks by itself).
    pub fn supporting(&self, op: Operation) {
        self.lock().off.retain(|o| *o != op);
    }

    /// `op` is not supported any more (a right taken back while a card is held).
    pub fn withdrawing(&self, op: Operation) {
        let mut m = self.lock();
        if !m.off.contains(&op) {
            m.off.push(op);
        }
    }

    /// The host opens no context menu.
    #[must_use]
    pub fn without_menu(self) -> Self {
        self.lock().no_menu = true;
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
    fn supports(&self, op: Operation) -> bool {
        !self.lock().off.contains(&op)
    }
    fn offers_menu(&self) -> bool {
        !self.lock().no_menu
    }
    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
    fn lane_plan(&self) -> LanePlan {
        self.lock().lanes.clone()
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
            return Answer::Ask(crate::requests::Question::new(
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
