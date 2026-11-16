//! Backspace in the typed tree: one character, or at the start of a block what a person expects of Notion and Confluence.
//!
//! Pattern: none — the key of a person applied to the tree.
//!
//! At the start of a block, in this order: a shortcut that has just fired is given back; a task box goes (the item stays); an
//! item or quoted paragraph leaves its list or quote; a heading or code block becomes a paragraph; a paragraph joins the leaf
//! above (an empty one only goes); a rule above goes.

use super::editor::{Cursor, RichEditor};
use super::node::Node;
use super::tree::Home;

fn here(path: &[usize], at: usize) -> Cursor {
    Cursor {
        path: path.to_vec(),
        at,
    }
}

impl RichEditor {
    /// Press Backspace at the cursor; the cursor afterwards.
    pub fn backspace(&mut self, c: &Cursor) -> Cursor {
        let Some(node) = self.node(&c.path) else {
            return c.clone();
        };
        let text = node.text().unwrap_or("");
        if c.at > 0 && c.at <= text.len() {
            let from = text[..c.at]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
            if let Some(t) = self.node_mut(&c.path).and_then(Node::text_mut) {
                t.replace_range(from..c.at, "");
            }
            self.touch(&c.path);
            self.fired = None;
            return here(&c.path, from);
        }
        if let Some(f) = self.fired.take().filter(|f| f.path == c.path) {
            return self.give_back(&c.path, &f.typed, f.box_only);
        }
        self.at_start(&c.path)
    }

    /// The shortcut is undone: the characters typed come back as text of a paragraph.
    fn give_back(&mut self, path: &[usize], typed: &str, box_only: bool) -> Cursor {
        let rest = self.text(path).to_owned();
        let text = format!("{typed}{rest}");
        let at = typed.len();
        let in_task_item = path.len() > 2 && self.item_of(path).is_some_and(|i| i.task.is_some());
        if box_only
            && in_task_item
            && self.home(path) == Home::Item
            && matches!(self.node(path), Some(Node::Para(_)))
        {
            self.set_task(path, None);
            if let Some(t) = self.node_mut(path).and_then(Node::text_mut) {
                *t = text;
            }
            return here(path, at);
        }
        // The block of the page the shortcut made (a heading, a list, a quote, a code block) is a paragraph again.
        if let Some(slot) = self.node_mut(&path[..1]) {
            *slot = Node::Para(text);
        }
        self.touch(path);
        here(&path[..1], at)
    }

    fn at_start(&mut self, path: &[usize]) -> Cursor {
        let home = self.home(path);
        let Some(node) = self.node(path).cloned() else {
            return here(path, 0);
        };
        match (&node, home) {
            (Node::Para(_), Home::Item) if self.item_of(path).is_some_and(|i| i.task.is_some()) => {
                self.set_task(path, None);
                here(path, 0)
            }
            (_, Home::Item | Home::Quote) if path.last() == Some(&0) || home == Home::Item => {
                match self.lift(path) {
                    Some(p) => here(&p, 0),
                    None => here(path, 0),
                }
            }
            (Node::Heading(_, t) | Node::Code(_, t), Home::Page) => {
                let t = t.clone();
                if let Some(slot) = self.node_mut(path) {
                    *slot = Node::Para(t);
                }
                self.touch(path);
                here(path, 0)
            }
            (Node::Rule, _) => self.remove_rule(path),
            _ => self.join_above(path, &node),
        }
    }

    /// A rule goes; the cursor lands in the leaf above, or in the one below when it was the first.
    fn remove_rule(&mut self, path: &[usize]) -> Cursor {
        let above = self.leaf_before(path);
        if path.len() == 1 {
            self.blocks.remove(path[0]);
        }
        if let Some(p) = above {
            let at = self.text(&p).len();
            return here(&p, at);
        }
        if self.blocks.is_empty() {
            self.blocks
                .push(super::block::RichBlock::new(Node::Para(String::new())));
        }
        here(&[0], 0)
    }

    /// A paragraph at the start of a block: it joins the leaf above; into code, a table or after a rule it only moves.
    fn join_above(&mut self, path: &[usize], node: &Node) -> Cursor {
        let Some(above) = self.leaf_before(path) else {
            return here(path, 0);
        };
        let empty = node.text().is_some_and(str::is_empty);
        let text = node.text().unwrap_or("").to_owned();
        match self.node(&above) {
            Some(Node::Rule) => {
                if above.len() == 1 {
                    self.blocks.remove(above[0]);
                    let mut p = path.to_vec();
                    p[0] -= 1;
                    return here(&p, 0);
                }
                here(path, 0)
            }
            Some(Node::Code(..) | Node::Raw(_)) => {
                let end = self.text(&above).len();
                if empty && path.len() == 1 {
                    self.blocks.remove(path[0]);
                }
                here(&above, end)
            }
            Some(Node::Para(_) | Node::Heading(..))
                if path.len() > 1
                    && matches!(node, Node::Para(_))
                    && above[..above.len() - 1] == path[..path.len() - 1] =>
            {
                let end = self.text(&above).len();
                let k = path[path.len() - 1];
                if let Some(c) = self.siblings_mut(path) {
                    c.remove(k);
                }
                if let Some(t) = self.node_mut(&above).and_then(Node::text_mut) {
                    t.push_str(&text);
                }
                self.touch(&above);
                here(&above, end)
            }
            Some(_) if path.len() == 1 && matches!(node, Node::Para(_)) => {
                let end = self.text(&above).len();
                self.blocks.remove(path[0]);
                if let Some(t) = self.node_mut(&above).and_then(Node::text_mut) {
                    t.push_str(&text);
                }
                self.touch(&above);
                here(&above, end)
            }
            _ => here(path, 0),
        }
    }
}
