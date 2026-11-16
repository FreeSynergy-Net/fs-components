//! The typed tree of one block: leaves with text, containers (quote, list, list item) that hold blocks.
//!
//! Pattern: Composite — a block is a leaf (paragraph, heading, code, rule, raw source) or a container of blocks (a quote, a list
//! of items, an item), exactly the shape of the container chain in `fs-text` (`Container::Quote`, `Container::Item`), but held
//! as a tree. A list is one container with its items; numbers are not stored per item but counted from `start`, so a new item
//! in the middle renumbers by being there. The kind is a property of the block, not a mark.

/// One block of the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Running text.
    Para(String),
    /// A heading of level 1 to 6.
    Heading(u8, String),
    /// A fenced code block: language hint (may be empty) and the lines between the fences.
    Code(String, String),
    /// A horizontal rule.
    Rule,
    /// Something the editor does not edit as a typed block (a table): kept and shown as source text.
    Raw(String),
    /// A quotation holding blocks.
    Quote(Vec<Node>),
    /// A list.
    List(List),
}

/// A list: bullets or numbers, and its items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List {
    /// Numbered (else bullets).
    pub ordered: bool,
    /// The number of the first item.
    pub start: u32,
    /// The items.
    pub items: Vec<Item>,
}

/// A list item: a task box if it has one, and the blocks it holds (the first is its text).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// `Some(done)` for a task item.
    pub task: Option<bool>,
    /// The blocks of the item.
    pub children: Vec<Node>,
}

impl Item {
    /// An item with one paragraph.
    #[must_use]
    pub fn text(task: Option<bool>, text: impl Into<String>) -> Self {
        Self {
            task,
            children: vec![Node::Para(text.into())],
        }
    }
}

impl List {
    /// A list of one item.
    #[must_use]
    pub fn one(ordered: bool, start: u32, item: Item) -> Self {
        Self {
            ordered,
            start,
            items: vec![item],
        }
    }
}

impl Node {
    /// The text of a leaf that holds text (none for a rule and for containers).
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Node::Para(t) | Node::Heading(_, t) | Node::Code(_, t) | Node::Raw(t) => Some(t),
            _ => None,
        }
    }

    /// The text of a leaf, to change.
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match self {
            Node::Para(t) | Node::Heading(_, t) | Node::Code(_, t) | Node::Raw(t) => Some(t),
            _ => None,
        }
    }

    /// The node at `path` below this one (a quote step is the child, a list step is the item and then its child).
    #[must_use]
    pub fn get(&self, path: &[usize]) -> Option<&Node> {
        let Some((&i, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Node::Quote(c) => c.get(i)?.get(rest),
            Node::List(l) => {
                let (&j, rest) = rest.split_first()?;
                l.items.get(i)?.children.get(j)?.get(rest)
            }
            _ => None,
        }
    }

    /// The node at `path`, to change.
    pub fn get_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let Some((&i, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Node::Quote(c) => c.get_mut(i)?.get_mut(rest),
            Node::List(l) => {
                let (&j, rest) = rest.split_first()?;
                l.items.get_mut(i)?.children.get_mut(j)?.get_mut(rest)
            }
            _ => None,
        }
    }

    /// The paths of the leaves below this node, in reading order.
    pub fn leaves(&self, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        let mut below = |node: &Node, steps: &[usize]| {
            path.extend_from_slice(steps);
            node.leaves(path, out);
            path.truncate(path.len() - steps.len());
        };
        match self {
            Node::Quote(c) => c.iter().enumerate().for_each(|(i, n)| below(n, &[i])),
            Node::List(l) => {
                for (i, item) in l.items.iter().enumerate() {
                    for (j, n) in item.children.iter().enumerate() {
                        below(n, &[i, j]);
                    }
                }
            }
            _ => out.push(path.clone()),
        }
    }

    /// Is this a leaf (text, rule) and not a container?
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        !matches!(self, Node::Quote(_) | Node::List(_))
    }
}
