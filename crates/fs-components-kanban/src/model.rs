//! The plain data of the board: columns, cards, the operations a host may support.
//!
//! Pattern: Value object — small types the host fills; no program type appears here, texts are
//! already in the reader's language (they are the host's data, not component words).

use fs_components_fields::{FieldSlot, Limit, SlotState};

use crate::limits::DEFAULT_FACTS;

/// Colour slot of a label (the style decides the colour).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    /// The accent colour.
    #[default]
    Accent,
    /// Success colour.
    Success,
    /// Error colour.
    Error,
    /// Neutral, weak.
    Neutral,
}

/// A label on a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    /// Text.
    pub text: String,
    /// Colour slot.
    pub tone: Tone,
}

/// How urgent a due date is; the host decides, the card only marks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DueState {
    /// Some time away.
    Open,
    /// Close.
    Soon,
    /// Past.
    Overdue,
    /// The card is done.
    Done,
}

/// The due mark of a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Due {
    /// The date as the reader should see it.
    pub text: String,
    /// Urgency.
    pub state: DueState,
}

/// One card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    /// Identity, handed back by every event.
    pub key: String,
    /// Title.
    pub title: String,
    /// Second line.
    pub subtitle: Option<String>,
    /// Labels.
    pub labels: Vec<Label>,
    /// Due mark.
    pub due: Option<Due>,
    /// Name of the assignee; the card shows initials.
    pub assignee: Option<String>,
    /// The facts line: field values, each drawn by the compact component of the block the catalog chose (slot `card-facts` of
    /// the block `board`). Empty = no line. Private: only [`Card::fact`] adds, and it checks the limit of the card.
    facts: Vec<FieldSlot>,
    /// How many facts the card takes: the setting of the instance (default [`DEFAULT_FACTS`]).
    max_facts: Limit,
    /// How many facts were handed in beyond the limit ([`Card::fact_or_more`]): the card says "+ N more", it never cuts silently.
    more_facts: usize,
}

impl Card {
    /// A card with a title and nothing else.
    pub fn new(key: &str, title: &str) -> Self {
        Self::limited(key, title, DEFAULT_FACTS)
    }

    /// A card that takes up to `max_facts` facts (the value the host hands in: the effective value of the setting `fs.kanban.card-facts`, see
    /// [`crate::FACTS_SETTING`]).
    pub fn limited(key: &str, title: &str, max_facts: Limit) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            subtitle: None,
            labels: Vec::new(),
            due: None,
            assignee: None,
            facts: Vec::new(),
            max_facts,
            more_facts: 0,
        }
    }

    /// The facts, in order.
    pub fn facts(&self) -> &[FieldSlot] {
        &self.facts
    }

    /// The facts that are DRAWN: a fact without a value ([`SlotState::Unset`]) takes no room on the card; a hidden or missing one
    /// shows its reason (the state decides, not the text).
    pub fn shown_facts(&self) -> impl Iterator<Item = &FieldSlot> {
        self.facts.iter().filter(|f| f.state != SlotState::Unset)
    }

    /// Whether the card has a facts line: a fact to draw, or the piece "+ N more".
    pub fn has_facts_line(&self) -> bool {
        self.more_facts > 0 || self.shown_facts().next().is_some()
    }

    /// How many facts the card takes.
    pub fn max_facts(&self) -> usize {
        self.max_facts.get()
    }

    /// How many facts did not fit (the card shows "+ N more").
    pub fn more_facts(&self) -> usize {
        self.more_facts
    }

    /// With one more fact, or counted as "more" when the card is full: nothing is cut silently (the view says "+ N more").
    #[must_use]
    pub fn fact_or_more(mut self, slot: FieldSlot) -> Self {
        if self.facts.len() >= self.max_facts.get() {
            self.more_facts += 1;
        } else {
            self.facts.push(slot);
        }
        self
    }

    /// With one more fact.
    ///
    /// # Errors
    /// The slot comes back when the card already has [`Card::max_facts`] facts: the caller says it is too many, nothing is dropped silently.
    pub fn fact(mut self, slot: FieldSlot) -> Result<Self, FieldSlot> {
        if self.facts.len() >= self.max_facts.get() {
            return Err(slot);
        }
        self.facts.push(slot);
        Ok(self)
    }
}

/// One column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// Identity.
    pub key: String,
    /// Title.
    pub title: String,
    /// Work-in-progress limit, if any.
    pub limit: Option<usize>,
    /// Cards, in order.
    pub cards: Vec<Card>,
}

impl Column {
    /// An empty column without a limit.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            limit: None,
            cards: Vec::new(),
        }
    }

    /// More cards than the limit allows.
    pub fn over_limit(&self) -> bool {
        self.limit.is_some_and(|l| self.cards.len() > l)
    }
}

/// What a board can be asked to do; the host says which it supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    /// Add a column.
    AddColumn,
    /// Rename a column.
    Rename,
    /// Set a column's limit.
    Limit,
    /// Move a card to another column.
    Move,
}

impl Operation {
    /// All operations.
    pub const ALL: [Operation; 4] = [
        Operation::AddColumn,
        Operation::Rename,
        Operation::Limit,
        Operation::Move,
    ];

    /// Name for translation keys and element ids.
    pub fn name(self) -> &'static str {
        match self {
            Operation::AddColumn => "add_column",
            Operation::Rename => "rename",
            Operation::Limit => "limit",
            Operation::Move => "move",
        }
    }
}
