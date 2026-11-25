//! The cursor and the keys of the grid state: where the cursor stands, how it moves, what a key does.
//!
//! Pattern: State (split from `model.rs`, same type; behaviour unchanged).

use super::hold::{Hold, hold};
use super::model::{Outcome, State};
use crate::keys::{Key, KeyEvent};

impl State {
    /// The cursor: the index among the VISIBLE rows and the column.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Put the cursor on this cell (the pointer's way); false when it is not there.
    pub fn go(&mut self, node: usize, col: usize) -> bool {
        let at = self.shown().iter().position(|s| s.node == node);
        match at {
            Some(row) if col <= self.last_col_of(node) => {
                self.set_cursor((row, col));
                self.want = col;
                true
            }
            _ => false,
        }
    }

    /// Move the cursor; the keys leave the cell they were in when the cursor leaves it.
    pub(super) fn set_cursor(&mut self, to: (usize, usize)) {
        if to != self.cursor {
            self.inside = false;
        }
        self.cursor = to;
    }

    /// The node with this key.
    pub fn find(&self, key: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.key == key)
    }

    /// Whether a node is open.
    pub fn is_open(&self, node: usize) -> bool {
        self.nodes[node].open
    }

    /// Open or close a parent row from the program; false when nothing changed.
    pub fn set_open(&mut self, node: usize, open: bool) -> bool {
        if self.nodes[node].children.is_empty() || self.nodes[node].open == open {
            return false;
        }
        let keep = self.shown().get(self.cursor.0).map(|s| s.node);
        self.nodes[node].open = open;
        // the cursor stays on its row when that is still shown, else it goes to the nearest shown ancestor
        let rows = self.shown();
        let mut want = keep;
        while let Some(n) = want {
            if let Some(at) = rows.iter().position(|s| s.node == n) {
                self.set_cursor((at, self.cursor.1));
                return true;
            }
            want = self.nodes[n].parent;
        }
        self.set_cursor((0, self.cursor.1));
        true
    }

    /// What a key does.
    pub fn press(&mut self, event: KeyEvent) -> Outcome {
        let rows = self.shown();
        let (row, col) = self.cursor;
        let (Some(last_row), Some(last_col)) =
            (rows.len().checked_sub(1), self.cols.len().checked_sub(1))
        else {
            return Outcome::Ignored;
        };
        let node = rows[row.min(last_row)].node;
        match hold(self.holds, self.inside, self.nodes[node].spans, event.key) {
            Hold::Normal => {}
            Hold::Enter => {
                self.inside = true;
                return Outcome::Entered(node, col);
            }
            Hold::Leave => {
                self.inside = false;
                return Outcome::Left(node, col);
            }
            Hold::Content => return Outcome::Ignored,
        }
        // along a row the aim is the new column; up and down keep the aim (a spanning row only has column 0)
        let across = |me: &mut Self, r: usize, c: usize| {
            me.want = c;
            if (r, c) == me.cursor {
                return Outcome::Ignored;
            }
            me.set_cursor((r, c));
            Outcome::Moved
        };
        let down_to = |me: &mut Self, r: usize| {
            let c = me.want.min(me.last_col_of(rows[r].node));
            if (r, c) == me.cursor {
                return Outcome::Ignored;
            }
            me.set_cursor((r, c));
            Outcome::Moved
        };
        let last_col = last_col.min(self.last_col_of(node));
        match event.key {
            Key::Right if col == 0 && rows[row].expanded == Some(false) => {
                self.nodes[node].open = true;
                Outcome::Opened(node)
            }
            Key::Left if col == 0 && rows[row].expanded == Some(true) => {
                self.nodes[node].open = false;
                Outcome::Closed(node)
            }
            Key::Right if col < last_col => across(self, row, col + 1),
            Key::Left if col > 0 => across(self, row, col - 1),
            Key::Down if row < last_row => down_to(self, row + 1),
            Key::Up if row > 0 => down_to(self, row - 1),
            Key::Home if event.ctrl => across(self, 0, 0),
            Key::End if event.ctrl => {
                let c = self.last_col_of(rows[last_row].node);
                across(self, last_row, c)
            }
            Key::Home => across(self, row, 0),
            Key::End => across(self, row, last_col),
            Key::PageDown => down_to(self, (row + self.page).min(last_row)),
            Key::PageUp => down_to(self, row.saturating_sub(self.page)),
            Key::Enter | Key::Space | Key::Activate => Outcome::Activated(node, col),
            _ => Outcome::Ignored,
        }
    }
}
