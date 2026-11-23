//! The state of the board: selection, cursor, the card held, and the moves.
//!
//! Pattern: State + Command — one machine for every way to move a card (grab/hover/drop of a
//! frontend gesture, keys, move buttons, click on a column). It knows no Blinc and does not move
//! anything: a finished move comes out as a [`MoveRequest`] that the host answers.

use crate::model::Column;

/// A card taken up and where it is held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The card.
    pub key: String,
    /// The column it came from.
    pub from: String,
    /// The column it is over now.
    pub over: String,
    /// Where it will land in `over`, counted WITHOUT the card itself (the number the host takes as `index`): `None` = at the end
    /// of another column, or, in its own column, where it is now (nothing to move).
    pub index: Option<usize>,
    /// Where the card stands in its own column when it was picked up.
    pub home: usize,
}

/// A move for the host to carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveRequest {
    /// The card.
    pub key: String,
    /// Column it leaves.
    pub from: String,
    /// Column it goes to.
    pub to: String,
    /// The place in `to`, counted without the card itself (the cursor of a drop between two cards: before card 1 and 2 is `Some(1)`);
    /// `None` = at the end. In the own column a request always has `Some` (a reorder).
    pub index: Option<usize>,
}

/// What a step came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Nothing happened.
    Ignored,
    /// The state changed; draw again.
    Changed,
    /// A card was chosen.
    Select(String),
    /// A move for the host.
    Move(MoveRequest),
    /// Refused here; the code is the tail of `kanban.refused.<code>`.
    Refused(&'static str),
}

/// Everything the board remembers.
#[derive(Clone, Debug, Default)]
pub struct BoardState {
    /// The chosen card.
    pub selected: Option<String>,
    /// The card under the keyboard cursor.
    pub cursor: Option<String>,
    /// The card held, if any.
    pub held: Option<Held>,
    /// Code of the last refusal, shown until the next step.
    pub notice: Option<String>,
}

/// Column and row of a card.
pub fn locate(cols: &[Column], key: &str) -> Option<(usize, usize)> {
    cols.iter()
        .enumerate()
        .find_map(|(c, col)| col.cards.iter().position(|k| k.key == key).map(|r| (c, r)))
}

impl BoardState {
    /// Choose a card.
    pub fn select(&mut self, key: &str) -> Step {
        self.selected = Some(key.to_owned());
        self.cursor = Some(key.to_owned());
        self.notice = None;
        Step::Select(key.to_owned())
    }

    /// Pick a card up (a gesture's start, or Space).
    pub fn grab(&mut self, cols: &[Column], can_move: bool, key: &str) -> Step {
        if !can_move {
            return self.refuse("locked");
        }
        let Some((c, home)) = locate(cols, key) else {
            return Step::Ignored;
        };
        self.held = Some(Held {
            key: key.to_owned(),
            from: cols[c].key.clone(),
            over: cols[c].key.clone(),
            index: None,
            home,
        });
        self.cursor = Some(key.to_owned());
        self.notice = None;
        Step::Changed
    }

    /// The held card is over `column`.
    pub fn hover(&mut self, cols: &[Column], column: &str) -> Step {
        match &mut self.held {
            Some(h) if cols.iter().any(|c| c.key == column) && h.over != column => {
                column.clone_into(&mut h.over);
                h.index = None;
                Step::Changed
            }
            _ => Step::Ignored,
        }
    }

    /// The held card is over `column`, in front of the card at `before` of that column (`None`: after the last one): the gesture
    /// that knows the card under the pointer. The place is worked out here, once, so the drop only reads it.
    pub fn hover_at(&mut self, cols: &[Column], column: &str, before: Option<usize>) -> Step {
        let (Some(h), Some(col)) = (self.held.as_mut(), cols.iter().find(|c| c.key == column))
        else {
            return Step::Ignored;
        };
        let len = col.cards.len();
        let p = before.map_or(len, |b| b.min(len));
        let index = if h.from == column {
            // Counted without the card itself; where it stands now is no move.
            let end = if p > h.home { p - 1 } else { p };
            (end != h.home).then_some(end)
        } else {
            (p < len).then_some(p)
        };
        if h.over == column && h.index == index {
            return Step::Ignored;
        }
        column.clone_into(&mut h.over);
        h.index = index;
        Step::Changed
    }

    /// Put the held card down where it is; a move when the column differs.
    /// `can_move` is asked again here: the host may have stopped supporting moves since the pick-up.
    pub fn drop_held(&mut self, can_move: bool) -> Step {
        match self.held.take() {
            Some(_) if !can_move => self.refuse("locked"),
            Some(h) if h.from != h.over || h.index.is_some() => Step::Move(MoveRequest {
                key: h.key,
                from: h.from,
                to: h.over,
                index: h.index,
            }),
            Some(_) => Step::Changed,
            None => Step::Ignored,
        }
    }

    /// Hold over `column` and put down there, in one step.
    pub fn drop_at(&mut self, cols: &[Column], can_move: bool, column: &str) -> Step {
        if self.held.is_none() {
            return Step::Ignored;
        }
        self.hover(cols, column);
        self.drop_held(can_move)
    }

    /// Give the held card up.
    pub fn cancel(&mut self) -> Step {
        if self.held.take().is_some() {
            Step::Changed
        } else {
            Step::Ignored
        }
    }

    /// Move a card one column left or right (`dir` -1 or 1): the move buttons.
    pub fn step_over(&mut self, cols: &[Column], can_move: bool, key: &str, dir: i32) -> Step {
        if !can_move {
            return self.refuse("locked");
        }
        let Some((c, _)) = locate(cols, key) else {
            return Step::Ignored;
        };
        let Some(to) = c.checked_add_signed(dir as isize).and_then(|i| cols.get(i)) else {
            return Step::Ignored;
        };
        self.notice = None;
        Step::Move(MoveRequest {
            key: key.to_owned(),
            from: cols[c].key.clone(),
            to: to.key.clone(),
            index: None,
        })
    }

    /// Move a card one place up (-1) or down (1) in its own column: the keys Alt+Up and Alt+Down (the frontend maps the keys; the
    /// board keys carry no modifier). At the first or last place nothing happens.
    pub fn reorder(&mut self, cols: &[Column], can_move: bool, key: &str, dir: i32) -> Step {
        if !can_move {
            return self.refuse("locked");
        }
        let Some((c, r)) = locate(cols, key) else {
            return Step::Ignored;
        };
        let Some(to) = r
            .checked_add_signed(dir as isize)
            .filter(|i| *i < cols[c].cards.len())
        else {
            return Step::Ignored;
        };
        self.notice = None;
        Step::Move(MoveRequest {
            key: key.to_owned(),
            from: cols[c].key.clone(),
            to: cols[c].key.clone(),
            index: Some(to),
        })
    }

    pub(crate) fn refuse(&mut self, code: &'static str) -> Step {
        self.notice = Some(code.to_owned());
        Step::Refused(code)
    }
}
