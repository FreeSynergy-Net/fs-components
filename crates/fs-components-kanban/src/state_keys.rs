//! The keyboard of the board: arrows, Shift+arrow, Space, Enter, Escape, N. What a key MEANS to a card (lift, put down, give up, open,
//! menu) is the one table of `card::keys` (`step`); here the board only walks the cursor (the container's part) and carries it out.
//! Ctrl is never taken: undo by key is `app.undo` of the host (one undo path, `Records::step_back`); the board only has the
//! "Undo" button of its notice.
//!
//! Pattern: State + Command (the keys of `BoardState`; split from `state.rs`).

use crate::requests::Reason;
use crate::source::BoardSource;
use crate::state::{Action, BoardState};
use fs_components_widgets::card::keys::{Mode, Step, step};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_widgets::sortable_list::{Command, Outcome, Refusal};
impl BoardState {
    /// A key. Arrows walk; Shift+arrow moves the card under the cursor by one step (no grabbing needed); Space holds and puts
    /// down; Enter opens, or puts down a held card; Escape gives up; N opens the quick entry of the cursor's column. Every key with Ctrl (Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y ...) is left
    /// to the host: `Action::None`, nothing is written (one undo path: `app.undo`).
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
            return Action::None;
        }
        if event.alt {
            return Action::None;
        }
        let mode = self.mode_of_cursor();
        match step(mode, event) {
            Step::Activate => {
                return self
                    .sorter
                    .cursor_key()
                    .map_or(Action::None, |k| Action::Open(k.to_owned()));
            }
            // a locked card asked to be lifted: the same road, which says why
            Step::Sort(Command::Toggle) | Step::Refuse => return self.toggle(source),
            Step::Sort(Command::Cancel) => return self.escape(),
            Step::Sort(walk) => return self.walk(walk),
            Step::Menu | Step::Pass => {}
        }
        // At rest the table leaves these to the container: the board walks, moves a step, opens the quick entry, gives up.
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
            Key::Activate => self
                .sorter
                .cursor_key()
                .map_or(Action::None, |k| Action::Open(k.to_owned())),
            Key::Other(78) if self.can_move => self
                .sorter
                .cursor()
                .and_then(|p| self.lane_key(p.lane))
                .map_or(Action::None, |c| Action::QuickOpen(c.to_owned())),
            _ => Action::None,
        }
    }

    /// How the card under the cursor stands, for the table of the keys: held by the keyboard, locked, or at rest.
    fn mode_of_cursor(&self) -> Mode {
        if self.sorter.grabbed().is_some() {
            Mode::Held
        } else if self
            .sorter
            .cursor_key()
            .is_some_and(|k| self.reasons.contains_key(k))
        {
            Mode::Locked
        } else {
            Mode::Rest
        }
    }

    pub(crate) fn escape(&mut self) -> Action {
        if self.sorter.grabbed().is_some() {
            self.sorter.cancel();
            self.grab_origin = None;
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

    pub(crate) fn walk(&mut self, command: Command) -> Action {
        match self.sorter.command(command) {
            Outcome::Refused(_) | Outcome::Ignored => Action::None,
            _ => Action::Redraw,
        }
    }

    pub(crate) fn toggle(&mut self, source: &dyn BoardSource) -> Action {
        if !self.can_move {
            if self.sorter.grabbed().is_none() {
                return Action::None;
            }
            return self.rights_gone().unwrap_or(Action::None);
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
                Action::Redraw
            }
            Outcome::Moved(m) => {
                // The held card was carried by arrows (the sorter moved it already); the same rules as for the pointer apply now.
                if let Err(reason) = source.can_drop(&m.item, &m.to.lane, None) {
                    self.load(source);
                    return self.refuse(&reason);
                }
                let (after_old, old_index) = origin.unwrap_or((before, m.from.index));
                self.finish(&m, old_index, after_old, (None, None), false)
            }
            Outcome::Refused(Refusal::Locked) => {
                let reason = self
                    .sorter
                    .cursor_key()
                    .and_then(|k| self.reasons.get(k).cloned())
                    .unwrap_or_else(|| Reason::new("locked", "This card cannot be moved."));
                self.refuse(&reason)
            }
            Outcome::Unchanged | Outcome::Cancelled | Outcome::Carried => Action::Redraw,
            _ => Action::None,
        }
    }

    /// Shift+arrow: one step of the card under the cursor (up or down in its column; left or right into the next column, at the
    /// same index or the end).
    pub(crate) fn step(&mut self, key: Key, source: &dyn BoardSource) -> Action {
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
