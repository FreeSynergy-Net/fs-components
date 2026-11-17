//! The state of the picker: the query, the answer of the source for it, who is chosen.
//!
//! Pattern: State machine, no Blinc. The state holds the source's answer for the CURRENT query and drops it with the next;
//! what is chosen is a list of [`PersonRef`]s and nothing else, so a name cannot be copied into it (the type has no room).

use std::sync::Arc;

use super::model::{CandidatePage, Outcome, PersonRef, PickerError, Refusal, Selection};
use super::source::CandidateSource;

/// What the list shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The source has not answered yet.
    Loading,
    /// The source answered.
    Ready(CandidatePage),
    /// The source could not answer.
    Failed(PickerError),
}

/// A line the person must see below the list, from "assign to me".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    /// The source knows no own card.
    NoMe,
    /// The source failed.
    Error(PickerError),
}

/// Query, answer, choice and cursor.
#[derive(Clone)]
pub struct Picking {
    pub(super) source: Arc<dyn CandidateSource>,
    pub(super) query: String,
    pub(super) phase: Phase,
    pub(super) chosen: Vec<PersonRef>,
    pub(super) cursor: Option<PersonRef>,
    pub(super) selection: Selection,
    pub(super) require_choice: bool,
    pub(super) limit: usize,
    pub(super) notice: Option<Notice>,
}

impl Picking {
    /// A picker over `source`: asks it at once for the first candidates.
    pub fn new(source: Arc<dyn CandidateSource>, limit: usize) -> Self {
        let mut me = Self {
            source,
            query: String::new(),
            phase: Phase::Loading,
            chosen: Vec::new(),
            cursor: None,
            selection: Selection::Multiple,
            require_choice: false,
            limit: limit.clamp(1, 50),
            notice: None,
        };
        me.ask();
        me
    }

    fn ask(&mut self) {
        self.phase = match self.source.search(&self.query, self.limit) {
            Ok(None) => Phase::Loading,
            Ok(Some(page)) => Phase::Ready(page),
            Err(e) => Phase::Failed(e),
        };
    }

    /// Start with these people chosen (kept as given, in order, without doubles).
    pub fn choose(&mut self, people: Vec<PersonRef>) {
        for p in people {
            if !self.chosen.contains(&p) {
                self.chosen.push(p);
            }
        }
        if self.selection == Selection::Single {
            self.chosen.truncate(1);
        }
    }

    /// One person or several.
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
        if selection == Selection::Single {
            self.chosen.truncate(1);
        }
    }

    /// Confirm only with somebody chosen.
    pub fn require_choice(&mut self, require: bool) {
        self.require_choice = require;
    }

    /// What the list shows now.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// The text typed.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The people chosen, in order.
    pub fn chosen(&self) -> &[PersonRef] {
        &self.chosen
    }

    /// The line from "assign to me", if there is one.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// The references of the rows shown (none while loading or failed).
    pub fn rows(&self) -> Vec<&PersonRef> {
        match &self.phase {
            Phase::Ready(page) => page.items.iter().map(|c| &c.reference).collect(),
            _ => Vec::new(),
        }
    }

    /// The row under the cursor (a shown row; the first if the cursor is not on one).
    pub fn cursor(&self) -> Option<&PersonRef> {
        let rows = self.rows();
        self.cursor
            .as_ref()
            .and_then(|c| rows.iter().copied().find(|r| *r == c))
            .or_else(|| rows.first().copied())
    }

    /// Ask the source again for the current query (the program calls this when a pending answer arrived).
    pub fn refresh(&mut self) -> Outcome {
        self.ask();
        Outcome::Changed
    }

    /// Narrow the list: a new query, a new ask.
    pub fn set_query(&mut self, text: &str) -> Outcome {
        self.query = text.to_string();
        self.notice = None;
        self.ask();
        Outcome::Changed
    }

    /// Choose or unchoose. A person who is chosen but not among the rows (an earlier choice, "assign to me") can be taken away.
    pub fn toggle(&mut self, person: &PersonRef) -> Outcome {
        let held = self.chosen.iter().position(|p| p == person);
        let shown = self.rows().contains(&person);
        self.notice = None;
        match (held, shown) {
            (Some(i), _) => {
                self.chosen.remove(i);
            }
            (None, true) => {
                if self.selection == Selection::Single {
                    self.chosen.clear();
                }
                self.chosen.push(person.clone());
            }
            (None, false) => return Outcome::Refused(Refusal::NoSuchCandidate),
        }
        if shown {
            self.cursor = Some(person.clone());
        }
        Outcome::Changed
    }

    /// "Assign to me": the source's own reference is chosen (in single mode it replaces the choice). No own card: the notice
    /// says so; a failing source: the notice shows its error. Never silent.
    pub fn choose_me(&mut self) -> Outcome {
        match self.source.me() {
            Ok(Some(me)) => {
                self.notice = None;
                if self.selection == Selection::Single {
                    self.chosen.clear();
                }
                if !self.chosen.contains(&me) {
                    self.chosen.push(me);
                }
                Outcome::Changed
            }
            Ok(None) => {
                self.notice = Some(Notice::NoMe);
                Outcome::Refused(Refusal::NoMe)
            }
            Err(e) => {
                self.notice = Some(Notice::Error(e));
                Outcome::Refused(Refusal::SourceFailed)
            }
        }
    }

    /// Move the cursor over the rows.
    pub fn move_cursor(&mut self, delta: isize) -> Outcome {
        let rows = self.rows();
        if rows.is_empty() {
            return Outcome::Refused(match self.phase {
                Phase::Ready(_) => Refusal::Empty,
                _ => Refusal::NotReady,
            });
        }
        let now = self
            .cursor()
            .and_then(|c| rows.iter().position(|r| *r == c))
            .unwrap_or(0);
        let to = now.saturating_add_signed(delta).min(rows.len() - 1);
        self.cursor = Some(rows[to].clone());
        Outcome::Changed
    }

    /// Done: the references chosen, also those the query hides.
    pub fn confirm(&self) -> Outcome {
        if self.require_choice && self.chosen.is_empty() {
            return Outcome::Refused(Refusal::NothingChosen);
        }
        Outcome::Confirmed(self.chosen.clone())
    }
}
