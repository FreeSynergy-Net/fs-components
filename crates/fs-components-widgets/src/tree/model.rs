//! The pure state of a tree.
//!
//! Pattern: State — the nodes in an arena (no references between them, only indices), the open marks, the cursor, the pick, and
//! what a key does; no element, no callback.

use crate::keys::Key;
use crate::tree_view::TreeNode;

struct Node {
    key: String,
    label: String,
    parent: Option<usize>,
    children: Vec<usize>,
    open: bool,
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the tree.
    Ignored,
    /// The cursor moved to this node.
    Moved(usize),
    /// This parent opened.
    Opened(usize),
    /// This parent closed.
    Closed(usize),
    /// This node was picked.
    Picked(usize),
}

/// One visible item as the view draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Index of the node.
    pub node: usize,
    /// The key the caller gave.
    pub key: String,
    /// The label.
    pub label: String,
    /// Depth, 1 for a top item (`aria-level`).
    pub level: usize,
    /// How many siblings it has, itself included (`aria-setsize`).
    pub set_size: usize,
    /// Its place among them, 1-based (`aria-posinset`).
    pub pos: usize,
    /// `Some(open)` for a parent, `None` for a leaf (`aria-expanded` is only set on parents).
    pub expanded: Option<bool>,
}

/// The nodes, the open marks, the cursor and the pick.
pub struct State {
    nodes: Vec<Node>,
    roots: Vec<usize>,
    cursor: Option<usize>,
    picked: Option<usize>,
}

impl State {
    /// A tree of these top items; a node whose `open` is set starts open. The cursor starts on the first item.
    pub fn new(top: &[TreeNode]) -> Self {
        let mut state = Self {
            nodes: Vec::new(),
            roots: Vec::new(),
            cursor: None,
            picked: None,
        };
        state.roots = top.iter().map(|n| state.add(n, None)).collect();
        state.cursor = state.roots.first().copied();
        state
    }

    fn add(&mut self, node: &TreeNode, parent: Option<usize>) -> usize {
        let at = self.nodes.len();
        self.nodes.push(Node {
            key: node.key.clone(),
            label: node.label.clone(),
            parent,
            children: Vec::new(),
            open: node.open && !node.children.is_empty(),
        });
        let children: Vec<usize> = node
            .children
            .iter()
            .map(|c| self.add(c, Some(at)))
            .collect();
        self.nodes[at].children = children;
        at
    }

    /// The visible items in order, top to bottom.
    pub fn rows(&self) -> Vec<Row> {
        let mut out = Vec::new();
        self.walk(&self.roots, 1, &mut out);
        out
    }

    fn walk(&self, siblings: &[usize], level: usize, out: &mut Vec<Row>) {
        for (p, &i) in siblings.iter().enumerate() {
            out.push(self.row(i, level, siblings.len(), p + 1));
            if self.nodes[i].open {
                self.walk(&self.nodes[i].children, level + 1, out);
            }
        }
    }

    /// The item for node `i` at `level`, the `pos`-th of `set_size` siblings.
    pub fn row(&self, i: usize, level: usize, set_size: usize, pos: usize) -> Row {
        let n = &self.nodes[i];
        Row {
            node: i,
            key: n.key.clone(),
            label: n.label.clone(),
            level,
            set_size,
            pos,
            expanded: (!n.children.is_empty()).then_some(n.open),
        }
    }

    /// The node the cursor is on.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// The node that was picked.
    pub fn picked(&self) -> Option<usize> {
        self.picked
    }

    /// The key of a node.
    pub fn key_of(&self, node: usize) -> &str {
        &self.nodes[node].key
    }

    /// The children of a node (the view draws them inside the node's `group`).
    pub fn children_of(&self, node: usize) -> &[usize] {
        &self.nodes[node].children
    }

    /// The top items.
    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    /// Whether a node is open.
    pub fn is_open(&self, node: usize) -> bool {
        self.nodes[node].open
    }

    /// The index of the node with this key.
    pub fn find(&self, key: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.key == key)
    }

    /// Open or close a parent from the program (a leaf stays as it is). The cursor is kept on a visible item: closing an ancestor of
    /// the cursor moves it to that ancestor.
    pub fn set_open(&mut self, node: usize, open: bool) -> bool {
        if self.nodes[node].children.is_empty() || self.nodes[node].open == open {
            return false;
        }
        self.nodes[node].open = open;
        if !open {
            self.cursor = self.cursor.map(|c| self.visible_ancestor(c));
        }
        true
    }

    /// The nearest node at or above `from` that is visible (no closed ancestor).
    fn visible_ancestor(&self, from: usize) -> usize {
        let mut shown = from;
        let mut at = from;
        while let Some(p) = self.nodes[at].parent {
            if !self.nodes[p].open {
                shown = p;
            }
            at = p;
        }
        shown
    }

    /// Pick a node (the pointer's way, and the cursor goes there).
    pub fn pick(&mut self, node: usize) {
        self.cursor = Some(node);
        self.picked = Some(node);
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        let Some(at) = self.cursor else {
            return Outcome::Ignored;
        };
        let rows = self.rows();
        let place = rows.iter().position(|r| r.node == at).unwrap_or(0);
        let go = |to: usize, me: &mut Self| {
            me.cursor = Some(rows[to].node);
            Outcome::Moved(rows[to].node)
        };
        match key {
            Key::Down if place + 1 < rows.len() => go(place + 1, self),
            Key::Up if place > 0 => go(place - 1, self),
            Key::Home if place > 0 => go(0, self),
            Key::End if place + 1 < rows.len() => go(rows.len() - 1, self),
            Key::Right => match rows[place].expanded {
                Some(false) => {
                    self.nodes[at].open = true;
                    Outcome::Opened(at)
                }
                Some(true) => {
                    let first = self.nodes[at].children[0];
                    self.cursor = Some(first);
                    Outcome::Moved(first)
                }
                None => Outcome::Ignored,
            },
            Key::Left => match (rows[place].expanded, self.nodes[at].parent) {
                (Some(true), _) => {
                    self.nodes[at].open = false;
                    Outcome::Closed(at)
                }
                (_, Some(parent)) => {
                    self.cursor = Some(parent);
                    Outcome::Moved(parent)
                }
                _ => Outcome::Ignored,
            },
            Key::Enter | Key::Space | Key::Activate => {
                self.picked = Some(at);
                Outcome::Picked(at)
            }
            _ => Outcome::Ignored,
        }
    }
}
