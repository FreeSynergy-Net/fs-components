//! Typing a character into the typed tree, and the shortcuts that fire on it.
//!
//! Pattern: none — the key of a person applied to the tree; the rules themselves are the table in [`super::rules`].

use super::editor::{Cursor, Fired, RichEditor};
use super::node::{Item, List, Node};
use super::rules::{self, Place, Turn};
use super::tree::Home;

impl RichEditor {
    /// Type `ch` at the cursor; the cursor afterwards. A Space that completes a shortcut turns the paragraph into its block.
    pub fn type_char(&mut self, c: &Cursor, ch: char) -> Cursor {
        let Some(node) = self.node_mut(&c.path) else {
            return c.clone();
        };
        let paragraph = matches!(node, Node::Para(_));
        let Some(text) = node.text_mut() else {
            return c.clone();
        };
        let at = c.at.min(text.len());
        text.insert(at, ch);
        let typed = text[..at + ch.len_utf8()].to_owned();
        let rest = text[at + ch.len_utf8()..].to_owned();
        self.touch(&c.path);
        self.fired = None;
        if ch == ' ' && paragraph {
            let place = match self.home(&c.path) {
                Home::Page => Place::Page,
                Home::Item if self.item_of(&c.path).is_some_and(|i| i.task.is_none()) => {
                    Place::Item
                }
                _ => Place::Inside,
            };
            if let Some(turn) = rules::on_space(&typed, place) {
                return self.turn(&c.path, turn, typed, rest);
            }
        }
        Cursor {
            path: c.path.clone(),
            at: at + ch.len_utf8(),
        }
    }

    /// Make the paragraph at `path` what `turn` says, keeping `rest` as its text; the cursor is at the start of that text.
    /// `typed` is what was typed for the shortcut: Backspace right after gives it back.
    pub(super) fn turn(
        &mut self,
        path: &[usize],
        turn: Turn,
        typed: String,
        rest: String,
    ) -> Cursor {
        let item = |task, text| Item::text(task, text);
        let mut at = path.to_vec();
        let (node, inner): (Node, Vec<usize>) = match turn {
            Turn::Heading(n) => (Node::Heading(n, rest), vec![]),
            Turn::Bullet => (
                Node::List(List::one(false, 1, item(None, rest))),
                vec![0, 0],
            ),
            Turn::Numbered(n) => (Node::List(List::one(true, n, item(None, rest))), vec![0, 0]),
            Turn::Task(done) if path.len() == 1 => (
                Node::List(List::one(false, 1, item(Some(done), rest))),
                vec![0, 0],
            ),
            Turn::Task(done) => {
                self.set_task(path, Some(done));
                if let Some(t) = self.node_mut(path).and_then(Node::text_mut) {
                    *t = rest;
                }
                self.fired = Some(Fired {
                    path: path.to_vec(),
                    typed,
                    box_only: true,
                });
                return Cursor { path: at, at: 0 };
            }
            Turn::Quote => (Node::Quote(vec![Node::Para(rest)]), vec![0]),
            Turn::Code(lang) => (Node::Code(lang, rest), vec![]),
            Turn::Rule => {
                if let Some(slot) = self.node_mut(path) {
                    *slot = Node::Rule;
                }
                self.touch(path);
                let below = self.insert_after(path, Node::Para(rest));
                return Cursor { path: below, at: 0 };
            }
        };
        if let Some(slot) = self.node_mut(path) {
            *slot = node;
        }
        self.touch(path);
        at.extend(inner);
        self.fired = Some(Fired {
            path: at.clone(),
            typed,
            box_only: false,
        });
        Cursor { path: at, at: 0 }
    }

    /// Put the task box state of the item the leaf at `path` is the text of.
    pub(super) fn set_task(&mut self, path: &[usize], task: Option<bool>) {
        let n = path.len();
        if n < 3 {
            return;
        }
        if let Some(Node::List(l)) = self.node_mut(&path[..n - 2])
            && let Some(item) = l.items.get_mut(path[n - 2])
        {
            item.task = task;
        }
        self.touch(path);
    }

    /// Check or uncheck the task item whose text the leaf at `path` is (a click on its box).
    pub fn toggle_task(&mut self, path: &[usize]) {
        if let Some(Item {
            task: Some(done), ..
        }) = self.item_of(path)
        {
            let now = !*done;
            self.set_task(path, Some(now));
            self.fired = None;
        }
    }
}
