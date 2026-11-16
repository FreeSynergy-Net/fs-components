//! Surgery on the typed tree: where a leaf stands, a new sibling, taking a leaf out of its quote or list item.
//!
//! Pattern: none — plain functions over the tree of [`RichEditor`]; the keys ([`super::keys`]) decide, these only do.

use super::block::RichBlock;
use super::editor::RichEditor;
use super::node::{List, Node};

/// Where a leaf stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Home {
    /// A block of the page.
    Page,
    /// A child of a quote.
    Quote,
    /// A child of a list item.
    Item,
}

impl RichEditor {
    pub(super) fn home(&self, path: &[usize]) -> Home {
        if path.len() < 2 {
            Home::Page
        } else if matches!(self.node(&path[..path.len() - 1]), Some(Node::Quote(_))) {
            Home::Quote
        } else {
            Home::Item
        }
    }

    /// The children of the quote or item the leaf at `path` stands in (not for a block of the page).
    pub(super) fn siblings_mut(&mut self, path: &[usize]) -> Option<&mut Vec<Node>> {
        let n = path.len();
        if self.home(path) == Home::Quote {
            match self.node_mut(&path[..n - 1])? {
                Node::Quote(c) => Some(c),
                _ => None,
            }
        } else {
            let i = *path.get(n.checked_sub(2)?)?;
            match self.node_mut(&path[..n - 2])? {
                Node::List(l) => Some(&mut l.items.get_mut(i)?.children),
                _ => None,
            }
        }
    }

    /// A new `node` right after the one at `path`, in the same place; the path of the new one.
    pub(super) fn insert_after(&mut self, path: &[usize], node: Node) -> Vec<usize> {
        self.insert_at(path, 1, node)
    }

    /// A new `node` right before the one at `path`; the path the old one has now.
    pub(super) fn insert_before(&mut self, path: &[usize], node: Node) -> Vec<usize> {
        self.insert_at(path, 0, node)
    }

    fn insert_at(&mut self, path: &[usize], shift: usize, node: Node) -> Vec<usize> {
        let mut new = path.to_vec();
        let last = new.len() - 1;
        new[last] += 1;
        let at = path[last] + shift;
        if path.len() == 1 {
            self.blocks.insert(at, RichBlock::new(node));
        } else if let Some(c) = self.siblings_mut(path) {
            c.insert(at, node);
            self.touch(path);
        }
        new
    }

    /// The blocks `before`, `mid` and `after` instead of block `b`; the index of the first of `mid`.
    pub(super) fn splice_block(
        &mut self,
        b: usize,
        before: Option<Node>,
        mid: Vec<Node>,
        after: Option<Node>,
    ) -> usize {
        let first = b + usize::from(before.is_some());
        let all = before.into_iter().chain(mid).chain(after);
        self.blocks.splice(b..=b, all.map(RichBlock::new));
        first
    }

    /// Take the leaf at `path` out of its quote or list item and make it a block of the page (a nested one stays where it is:
    /// none). The rest of the quote or list stays on either side. The path of the leaf now.
    pub(super) fn lift(&mut self, path: &[usize]) -> Option<Vec<usize>> {
        let b = path[0];
        match (self.home(path), self.blocks.get(b)?.node.clone()) {
            (Home::Quote, Node::Quote(mut c)) if path.len() == 2 => {
                let k = path[1];
                let node = c.remove(k);
                let after = c.split_off(k);
                let before = (!c.is_empty()).then_some(Node::Quote(c));
                let after = (!after.is_empty()).then_some(Node::Quote(after));
                Some(vec![self.splice_block(b, before, vec![node], after)])
            }
            (Home::Item, Node::List(mut l)) if path.len() == 3 => {
                let k = path[1];
                let after = l.items.split_off(k + 1);
                let item = l.items.pop()?;
                let before = (!l.items.is_empty()).then_some(Node::List(l.clone()));
                let after = (!after.is_empty()).then(|| {
                    Node::List(List {
                        ordered: l.ordered,
                        start: l.start.saturating_add(u32::try_from(k + 1).unwrap_or(0)),
                        items: after,
                    })
                });
                Some(vec![self.splice_block(b, before, item.children, after)])
            }
            _ => None,
        }
    }

    /// The path of the leaf before `path` in reading order, if any.
    pub(super) fn leaf_before(&self, path: &[usize]) -> Option<Vec<usize>> {
        let all = self.leaves();
        let k = all.iter().position(|p| p == path)?;
        all.get(k.checked_sub(1)?).cloned()
    }

    /// The path of the leaf after `path` in reading order, if any.
    pub(super) fn leaf_after(&self, path: &[usize]) -> Option<Vec<usize>> {
        let all = self.leaves();
        let k = all.iter().position(|p| p == path)?;
        all.get(k + 1).cloned()
    }
}
