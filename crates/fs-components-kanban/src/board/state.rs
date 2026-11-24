//! The state of the board: heads and cards, the cursor and the card held (the `sortable-list` state machine), the pointer gesture,
//! the notice with its undo, the question of the program, the open quick entry and the open card.
//!
//! Pattern: State + Command, pure - no Blinc, no clock, no thread. Every way to move a card (the pointer, Shift+arrow, the held
//! card of the keyboard, a command of the context menu, undo) ends in [`BoardState::place`], so all of them produce the same
//! [`MoveRequest`] and meet the same rules (the program's `can_drop`, a locked card, the counted index and `after`). The state writes
//! nothing: it draws optimistically (the card is at its new place at once) and the program's answer keeps it or sets it back.

use std::collections::HashMap;

use crate::{Card, Column};
use fs_components_widgets::sortable_list::{Item, Lane, Sorter};

use super::drag::Drag;
use super::hit::{Geometry, Spot};
use super::model::{MoveRequest, Question, Reason};
use super::source::BoardSource;
use super::wip::{WipMark, wip_mark};

/// A column head: what the board knows of a column besides its cards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Head {
    /// The column key.
    pub key: String,
    /// The title.
    pub title: String,
    /// The work-in-progress limit.
    pub limit: Option<usize>,
    /// Cards the filter hides.
    pub hidden: usize,
}

/// The line under the board: something happened, and maybe it can be taken back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The sentence (the program's reason, or the board's word for "moved").
    pub text: String,
    /// The request that takes the last move back, if there is one.
    pub undo: Option<MoveRequest>,
    /// A refusal (drawn as a warning), not a plain remark.
    pub refused: bool,
}

/// What a gesture or a key came to; the workbench carries it out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing to do.
    None,
    /// Only the picture changes.
    Redraw,
    /// Open this card.
    Open(String),
    /// Ask the program to move; the second request takes the move back.
    Move(MoveRequest, MoveRequest),
    /// Open the quick entry at the foot of this column.
    QuickOpen(String),
    /// Take the last move back.
    Undo,
}

/// Everything the board remembers.
#[derive(Clone, Debug, Default)]
pub struct BoardState {
    pub(super) heads: Vec<Head>,
    pub(super) cards: HashMap<String, Card>,
    pub(super) reasons: HashMap<String, Reason>,
    pub(super) sorter: Sorter,
    pub(super) drag: Drag,
    pub(super) geometry: Geometry,
    pub(super) blocked: Option<Reason>,
    pub(super) notice: Option<Notice>,
    pub(super) question: Option<(MoveRequest, Question)>,
    pub(super) quick: Option<String>,
    pub(super) open: Option<String>,
    pub(super) generation: u64,
    pub(super) can_move: bool,
    pub(super) grab_origin: Option<(Option<String>, usize)>,
}

impl BoardState {
    /// A state loaded from `source`.
    pub fn new(source: &dyn BoardSource) -> Self {
        let mut s = Self::default();
        s.load(source);
        s
    }

    /// Read everything from `source` again (the data changed, or a move was not accepted). The cursor stays with its card.
    pub fn load(&mut self, source: &dyn BoardSource) {
        let columns: Vec<Column> = source.columns();
        self.heads = columns
            .iter()
            .map(|c| Head {
                key: c.key.clone(),
                title: c.title.clone(),
                limit: c.limit,
                hidden: source.hidden(&c.key),
            })
            .collect();
        self.cards.clear();
        self.reasons.clear();
        let mut lanes = Vec::new();
        for c in &columns {
            let mut items = Vec::new();
            for card in &c.cards {
                let mut item = Item::new(card.key.clone(), card.title.clone());
                if let Some(reason) = source.locked(&card.key) {
                    item = item.locked();
                    self.reasons.insert(card.key.clone(), reason);
                }
                items.push(item);
                self.cards.insert(card.key.clone(), card.clone());
            }
            lanes.push(Lane::new(c.key.clone(), c.title.clone(), items));
        }
        self.sorter.set_lanes(lanes);
        self.generation = source.generation();
        self.can_move = source.supports(crate::Operation::Move);
        self.drag = Drag::Idle;
        self.blocked = None;
        self.geometry = self.fresh_geometry();
        if self
            .open
            .as_ref()
            .is_some_and(|k| !self.cards.contains_key(k))
        {
            self.open = None;
        }
    }

    /// The source says its data changed since the last load.
    pub fn is_stale(&self, source: &dyn BoardSource) -> bool {
        source.generation() != self.generation
    }

    pub(super) fn fresh_geometry(&self) -> Geometry {
        Geometry::new(
            self.sorter
                .lanes()
                .iter()
                .map(|l| l.items.iter().map(|i| i.key.clone()).collect())
                .collect(),
        )
    }

    /// The heads, in order.
    pub fn heads(&self) -> &[Head] {
        &self.heads
    }

    /// The lanes as they are now (also while a card is carried by the keyboard).
    pub fn lanes(&self) -> &[Lane] {
        self.sorter.lanes()
    }

    /// The card with this key.
    pub fn card(&self, key: &str) -> Option<&Card> {
        self.cards.get(key)
    }

    /// Why a card is locked, if it is.
    pub fn locked(&self, key: &str) -> Option<&Reason> {
        self.reasons.get(key)
    }

    /// The rectangles of the shown columns.
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// The mark of column `n`: counts every card of it, also the hidden ones; warns above the limit, never locks.
    pub fn wip(&self, n: usize) -> Option<WipMark> {
        let head = self.heads.get(n)?;
        let shown = self.sorter.lanes().get(n)?.items.len();
        Some(wip_mark(shown + head.hidden, head.limit))
    }

    /// The card under the keyboard cursor.
    pub fn cursor(&self) -> Option<&str> {
        self.sorter.cursor_key()
    }

    /// The card held by the keyboard.
    pub fn held(&self) -> Option<&str> {
        self.sorter.grabbed()
    }

    /// The card carried by the pointer, and where it would land (`None` = over no column).
    pub fn carried(&self) -> Option<(&str, Option<Spot>, (f32, f32))> {
        match &self.drag {
            Drag::Dragging {
                card,
                over,
                pointer,
                ..
            } => Some((card.as_str(), *over, *pointer)),
            _ => None,
        }
    }

    /// Why the place the carried card is over does not take it, if it does not.
    pub fn blocked(&self) -> Option<&Reason> {
        self.blocked.as_ref()
    }

    /// The line under the board.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// Take the notice away.
    pub fn clear_notice(&mut self) {
        self.notice = None;
    }

    /// The program's question and the request it is about.
    pub fn question(&self) -> Option<&(MoveRequest, Question)> {
        self.question.as_ref()
    }

    /// The column whose foot holds an open quick entry.
    pub fn quick(&self) -> Option<&str> {
        self.quick.as_deref()
    }

    /// Open the quick entry at the foot of `column` (or close it with `None`).
    pub fn set_quick(&mut self, column: Option<&str>) {
        self.quick = column.map(str::to_owned);
    }

    /// The card shown in the frame at the side.
    pub fn open_card(&self) -> Option<&str> {
        self.open.as_deref()
    }

    /// Show a card in the frame at the side (`None` closes it).
    pub fn set_open(&mut self, card: Option<&str>) {
        self.open = card.map(str::to_owned);
    }

    /// The column key of lane `n`.
    pub(super) fn lane_key(&self, n: usize) -> Option<&str> {
        self.sorter.lanes().get(n).map(|l| l.key.as_str())
    }

    // ---- the pointer ------------------------------------------------------------------------------------------------------

    // ---- moving -----------------------------------------------------------------------------------------------------------

    // ---- the keyboard -----------------------------------------------------------------------------------------------------
}
