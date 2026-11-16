//! Enter in the typed tree: split a block, carry a list on, leave a list or a quote, open a code block.
//!
//! Pattern: none — the key of a person applied to the tree.

use super::block::RichBlock;
use super::editor::{Cursor, Fired, RichEditor};
use super::node::{Item, Node};
use super::rules::{self, Turn};
use super::tree::Home;

fn here(path: &[usize], at: usize) -> Cursor {
    Cursor {
        path: path.to_vec(),
        at,
    }
}

impl RichEditor {
    /// Press Enter at the cursor; the cursor afterwards.
    pub fn enter(&mut self, c: &Cursor) -> Cursor {
        self.fired = None;
        let Some(node) = self.node(&c.path).cloned() else {
            return c.clone();
        };
        let at = c.at.min(node.text().map_or(0, str::len));
        match node {
            Node::Code(_, text) => self.enter_in_code(c, &text, at),
            Node::Rule => self.paragraph_after(&c.path),
            Node::Raw(text) => self.split_raw(&c.path, &text, at),
            Node::Para(text) | Node::Heading(_, text) => self.enter_in_text(c, &text, at),
            Node::Quote(_) | Node::List(_) => c.clone(),
        }
    }

    fn paragraph_after(&mut self, path: &[usize]) -> Cursor {
        let below = self.insert_after(path, Node::Para(String::new()));
        here(&below, 0)
    }

    /// Enter in code is a line break; on the empty last line it leaves the block.
    fn enter_in_code(&mut self, c: &Cursor, text: &str, at: usize) -> Cursor {
        if at == text.len() && text.ends_with('\n') {
            if let Some(t) = self.node_mut(&c.path).and_then(Node::text_mut) {
                t.pop();
            }
            self.touch(&c.path);
            return self.paragraph_after(&c.path);
        }
        if let Some(t) = self.node_mut(&c.path).and_then(Node::text_mut) {
            t.insert(at, '\n');
        }
        self.touch(&c.path);
        here(&c.path, at + 1)
    }

    /// Enter in a table or other source text splits the source in two blocks (only for a block of the page).
    fn split_raw(&mut self, path: &[usize], text: &str, at: usize) -> Cursor {
        if path.len() != 1 {
            return here(path, at);
        }
        let (left, right) = (RichBlock::parse(&text[..at]), RichBlock::parse(&text[at..]));
        self.blocks.splice(path[0]..=path[0], [left, right]);
        here(&[path[0] + 1], 0)
    }

    fn enter_in_text(&mut self, c: &Cursor, text: &str, at: usize) -> Cursor {
        let home = self.home(&c.path);
        let is_para = matches!(self.node(&c.path), Some(Node::Para(_)));
        if home == Home::Page && is_para && at == text.len() {
            match rules::on_enter(text) {
                Some(Turn::Code(lang)) => {
                    let typed = text.to_owned();
                    if let Some(slot) = self.node_mut(&c.path) {
                        *slot = Node::Code(lang, String::new());
                    }
                    self.touch(&c.path);
                    self.fired = Some(Fired {
                        path: c.path.clone(),
                        typed,
                        box_only: false,
                    });
                    return here(&c.path, 0);
                }
                Some(Turn::Rule) => {
                    if let Some(slot) = self.node_mut(&c.path) {
                        *slot = Node::Rule;
                    }
                    self.touch(&c.path);
                    return self.paragraph_after(&c.path);
                }
                _ => {}
            }
        }
        if text.is_empty() && home != Home::Page {
            // An empty item or quoted paragraph leaves the list or the quote.
            if let Some(path) = self.lift(&c.path) {
                return here(&path, 0);
            }
        }
        if at == 0 && !text.is_empty() {
            return self.empty_before(&c.path, home);
        }
        self.split_at(&c.path, home, at)
    }

    /// Enter at the start of a text that is not empty: an empty block (or item) goes above, the cursor stays with the text.
    fn empty_before(&mut self, path: &[usize], home: Home) -> Cursor {
        let node = Node::Para(String::new());
        let new = match home {
            Home::Item if self.item_of(path).is_some() => {
                let n = path.len();
                let task = self.item_of(path).and_then(|i| i.task).map(|_| false);
                if let Some(Node::List(l)) = self.node_mut(&path[..n - 2]) {
                    l.items.insert(path[n - 2], Item::text(task, ""));
                }
                self.touch(path);
                let mut p = path.to_vec();
                p[n - 2] += 1;
                p
            }
            _ => self.insert_before(path, node),
        };
        here(&new, 0)
    }

    /// Split the text at `at`: the rest is a new item, paragraph or quoted paragraph below.
    fn split_at(&mut self, path: &[usize], home: Home, at: usize) -> Cursor {
        let rest = self
            .node_mut(path)
            .and_then(Node::text_mut)
            .map(|t| t.split_off(at))
            .unwrap_or_default();
        self.touch(path);
        let n = path.len();
        let item_text = home == Home::Item && path[n - 1] == 0 && self.item_of(path).is_some();
        if item_text {
            let task = self.item_of(path).and_then(|i| i.task).map(|_| false);
            if let Some(Node::List(l)) = self.node_mut(&path[..n - 2]) {
                l.items.insert(path[n - 2] + 1, Item::text(task, rest));
            }
            let mut p = path.to_vec();
            p[n - 2] += 1;
            return here(&p, 0);
        }
        let below = self.insert_after(path, Node::Para(rest));
        here(&below, 0)
    }
}
