//! The state of the board: heads and cards, the cursor and the card held (the `sortable-list` state machine), the pointer gesture,
//! the notice with its undo, the question of the program, the open quick entry and the open card.
//!
//! Pattern: State + Command, pure - no Blinc, no clock, no thread. Every way to move a card (the pointer, Shift+arrow, the held
//! card of the keyboard, a command of the context menu, undo) ends in [`BoardState::place`], so all of them produce the same
//! [`MoveRequest`] and meet the same rules (the program's `can_drop`, a locked card, the counted index and `after`). The state writes
//! nothing: it draws optimistically (the card is at its new place at once) and the program's answer keeps it or sets it back.

use std::collections::HashMap;

use fs_components_kanban::{Card, Column};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_widgets::sortable_list::{
    Command, Item, Lane, Moved, Outcome, Place, Refusal, Sorter,
};

use super::drag::{Drag, Moved as Pointer, Release};
use super::hit::{Geometry, Spot};
use super::model::{Answer, MoveRequest, Question, Reason};
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
    heads: Vec<Head>,
    cards: HashMap<String, Card>,
    reasons: HashMap<String, Reason>,
    sorter: Sorter,
    drag: Drag,
    geometry: Geometry,
    blocked: Option<Reason>,
    notice: Option<Notice>,
    question: Option<(MoveRequest, Question)>,
    quick: Option<String>,
    open: Option<String>,
    generation: u64,
    can_move: bool,
    grab_origin: Option<(Option<String>, usize)>,
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
        self.can_move = source.supports().reorder;
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

    fn fresh_geometry(&self) -> Geometry {
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
    fn lane_key(&self, n: usize) -> Option<&str> {
        self.sorter.lanes().get(n).map(|l| l.key.as_str())
    }

    // ---- the pointer ------------------------------------------------------------------------------------------------------

    /// The button went down on `card` at `window` (window coordinates).
    pub fn press(&mut self, card: &str, window: (f32, f32)) {
        if !self.can_move {
            return;
        }
        let Some(from) = self
            .sorter
            .find(card)
            .and_then(|p| self.lane_key(p.lane))
            .map(str::to_owned)
        else {
            return;
        };
        self.drag.press(card, &from, window);
    }

    /// The pointer moved: `window` for the 4 px rule, `local` in the columns row for the place.
    pub fn pointer(
        &mut self,
        window: (f32, f32),
        local: (f32, f32),
        source: &dyn BoardSource,
    ) -> Pointer {
        let moved = self.drag.pointer(window, local, &self.geometry);
        if let Drag::Dragging { card, over, .. } = &self.drag {
            // A locked card is not carried; the reason is said at once.
            if let Some(reason) = self.reasons.get(card).cloned() {
                self.blocked = Some(reason.clone());
                return moved;
            }
            self.blocked = over
                .and_then(|spot| self.lane_key(spot.column))
                .and_then(|column| source.can_drop(card, column).err());
        }
        moved
    }

    /// The button went up.
    pub fn release(&mut self, source: &dyn BoardSource) -> Action {
        self.blocked = None;
        match self.drag.release() {
            Release::Nothing | Release::Back => Action::Redraw,
            Release::Click(card) => Action::Open(card),
            Release::Drop { card, spot, .. } => {
                let Some(to) = self.lane_key(spot.column).map(str::to_owned) else {
                    return Action::Redraw;
                };
                self.place(&card, &to, spot.index, source, false)
            }
        }
    }

    // ---- moving -----------------------------------------------------------------------------------------------------------

    /// Put `card` at `index` of `column` (counted without the card itself) - the one road every move takes. A locked card or a
    /// column that does not take it gives a notice and no request; putting a card where it is gives nothing.
    pub fn place(
        &mut self,
        card: &str,
        column: &str,
        index: usize,
        source: &dyn BoardSource,
        confirmed: bool,
    ) -> Action {
        if let Some(reason) = self.reasons.get(card).cloned() {
            return self.refuse(reason);
        }
        if let Err(reason) = source.can_drop(card, column) {
            return self.refuse(reason);
        }
        let Some(before) = self.sorter.find(card) else {
            return Action::None;
        };
        let after_old = self.predecessor(card);
        match self.sorter.move_item(card, &Place::new(column, index)) {
            Outcome::Moved(m) => self.finish(&m, before.index, after_old, confirmed),
            Outcome::Unchanged => Action::Redraw,
            Outcome::Refused(Refusal::LaneClosed) => self.refuse(Reason::new(
                "closed",
                "This column does not take cards from elsewhere.",
            )),
            _ => Action::None,
        }
    }

    /// The sorter has moved the card (optimistic): the request for the program and the request that takes it back.
    fn finish(
        &mut self,
        m: &Moved,
        old_index: usize,
        after_old: Option<String>,
        confirmed: bool,
    ) -> Action {
        let after = self.predecessor(&m.item);
        self.geometry = self.fresh_geometry();
        let request = MoveRequest {
            card: m.item.clone(),
            from: m.from.lane.clone(),
            to: m.to.lane.clone(),
            index: m.to.index,
            after,
            confirmed,
        };
        let undo = MoveRequest {
            card: m.item.clone(),
            from: m.to.lane.clone(),
            to: m.from.lane.clone(),
            index: old_index,
            after: after_old,
            confirmed: true,
        };
        Action::Move(request, undo)
    }

    fn refuse(&mut self, reason: Reason) -> Action {
        self.notice = Some(Notice {
            text: reason.text,
            undo: None,
            refused: true,
        });
        Action::Redraw
    }

    /// The program answered a move. `undo` is the request that takes it back (made by the workbench before the move).
    pub fn answered(
        &mut self,
        request: &MoveRequest,
        answer: Answer,
        undo: Option<MoveRequest>,
        moved_text: &str,
        source: &dyn BoardSource,
    ) {
        match answer {
            Answer::Accepted => {
                self.question = None;
                self.notice = Some(Notice {
                    text: moved_text.to_owned(),
                    undo,
                    refused: false,
                });
                self.load(source);
            }
            Answer::Refused(reason) => {
                self.question = None;
                self.notice = Some(Notice {
                    text: reason.text,
                    undo: None,
                    refused: true,
                });
                self.load(source);
            }
            Answer::Ask(question) => {
                self.question = Some((request.clone(), question));
                self.load(source);
            }
        }
    }

    /// The reader answered the program's question.
    pub fn take_question(&mut self) -> Option<(MoveRequest, Question)> {
        self.question.take()
    }

    /// The card before `card` in its column, as shown now (what `undo_of` needs, asked before a move).
    pub fn predecessor(&self, card: &str) -> Option<String> {
        let at = self.sorter.find(card)?;
        at.index
            .checked_sub(1)
            .and_then(|i| self.sorter.lanes()[at.lane].items.get(i))
            .map(|i| i.key.clone())
    }

    // ---- the keyboard -----------------------------------------------------------------------------------------------------

    /// A key. Arrows walk; Shift+arrow moves the card under the cursor by one step (no grabbing needed); Space holds and puts
    /// down; Enter opens, or puts down a held card; Escape gives up; N opens the quick entry of the cursor's column; Ctrl+Z undoes.
    pub fn key(&mut self, event: &KeyEvent, source: &dyn BoardSource) -> Action {
        if self.quick.is_some() {
            return if event.key == Key::Escape {
                self.quick = None;
                Action::Redraw
            } else {
                Action::None
            };
        }
        if event.ctrl {
            return match event.key {
                Key::Other(90) => Action::Undo,
                _ => Action::None,
            };
        }
        if event.alt {
            return Action::None;
        }
        match event.key {
            Key::Escape => self.escape(),
            Key::Up | Key::Down | Key::Left | Key::Right if event.shift => {
                self.step(event.key, source)
            }
            Key::Up => self.walk(Command::Up),
            Key::Down => self.walk(Command::Down),
            Key::Left => self.walk(Command::Left),
            Key::Right => self.walk(Command::Right),
            Key::Home => self.walk(Command::First),
            Key::End => self.walk(Command::Last),
            Key::PageUp => self.walk(Command::PageUp),
            Key::PageDown => self.walk(Command::PageDown),
            Key::Space => self.toggle(source),
            Key::Enter | Key::Activate => {
                if self.sorter.grabbed().is_some() {
                    self.toggle(source)
                } else {
                    self.sorter
                        .cursor_key()
                        .map_or(Action::None, |k| Action::Open(k.to_owned()))
                }
            }
            Key::Other(78) if self.can_move => self
                .sorter
                .cursor()
                .and_then(|p| self.lane_key(p.lane))
                .map_or(Action::None, |c| Action::QuickOpen(c.to_owned())),
            _ => Action::None,
        }
    }

    fn escape(&mut self) -> Action {
        if self.drag.cancel() {
            self.blocked = None;
            return Action::Redraw;
        }
        if self.sorter.grabbed().is_some() {
            self.sorter.cancel();
            self.grab_origin = None;
            self.geometry = self.fresh_geometry();
            return Action::Redraw;
        }
        if self.open.take().is_some()
            || self.notice.take().is_some()
            || self.question.take().is_some()
        {
            return Action::Redraw;
        }
        Action::None
    }

    fn walk(&mut self, command: Command) -> Action {
        match self.sorter.command(command) {
            Outcome::Refused(_) | Outcome::Ignored => Action::None,
            _ => {
                self.geometry = self.fresh_geometry();
                Action::Redraw
            }
        }
    }

    fn toggle(&mut self, source: &dyn BoardSource) -> Action {
        if !self.can_move {
            return Action::None;
        }
        let holding = self.sorter.grabbed().map(str::to_owned);
        let before = holding.as_deref().and_then(|k| self.predecessor(k));
        let origin = self.grab_origin.take();
        match self.sorter.command(Command::Toggle) {
            Outcome::Grabbed(card) => {
                self.grab_origin = Some((
                    self.predecessor(&card),
                    self.sorter.find(&card).map_or(0, |p| p.index),
                ));
                self.geometry = self.fresh_geometry();
                Action::Redraw
            }
            Outcome::Moved(m) => {
                // The held card was carried by arrows (the sorter moved it already); the same rules as for the pointer apply now.
                if let Err(reason) = source.can_drop(&m.item, &m.to.lane) {
                    self.load(source);
                    return self.refuse(reason);
                }
                let (after_old, old_index) = origin.unwrap_or((before, m.from.index));
                self.finish(&m, old_index, after_old, false)
            }
            Outcome::Refused(Refusal::Locked) => {
                let reason = self
                    .sorter
                    .cursor_key()
                    .and_then(|k| self.reasons.get(k).cloned())
                    .unwrap_or_else(|| Reason::new("locked", "This card cannot be moved."));
                self.refuse(reason)
            }
            Outcome::Unchanged | Outcome::Cancelled | Outcome::Carried => {
                self.geometry = self.fresh_geometry();
                Action::Redraw
            }
            _ => Action::None,
        }
    }

    /// Shift+arrow: one step of the card under the cursor (up or down in its column; left or right into the next column, at the
    /// same index or the end).
    fn step(&mut self, key: Key, source: &dyn BoardSource) -> Action {
        if !self.can_move || self.sorter.grabbed().is_some() {
            return Action::None;
        }
        let Some(at) = self.sorter.cursor() else {
            return Action::None;
        };
        let lanes = self.sorter.lanes();
        let len = lanes[at.lane].items.len();
        let Some(card) = lanes[at.lane].items.get(at.index).map(|i| i.key.clone()) else {
            return Action::None;
        };
        let (lane, index) = match key {
            Key::Up if at.index > 0 => (at.lane, at.index - 1),
            Key::Down if at.index + 1 < len => (at.lane, at.index + 1),
            Key::Left if at.lane > 0 => (at.lane - 1, at.index),
            Key::Right if at.lane + 1 < lanes.len() => (at.lane + 1, at.index),
            _ => return Action::None,
        };
        let column = lanes[lane].key.clone();
        let index = index.min(lanes[lane].items.len());
        self.place(&card, &column, index, source, false)
    }
}
