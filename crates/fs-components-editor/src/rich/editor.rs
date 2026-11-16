//! The rich editor: the typed tree of the blocks, and the contract of the editor building blocks on top of it.
//!
//! Pattern: Strategy (the third editor behind [`TextEditor`]) over an Adapter ([`RichBlock`]): a document of Markdown source
//! blocks is read into typed blocks; typing and keys change the typed blocks (the keys are in `type_in`, `enter` and `back`, the
//! tree surgery in `tree`); what goes out is only block operations of the same model. A block nobody touched keeps its
//! source byte for byte.

use super::block::RichBlock;
use super::node::{Item, Node};
use crate::{Doc, Edit, Op, TextEditor, diff};

/// A place in the text of a leaf: the path of the leaf and a byte offset into its text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    /// The path of the leaf: the block, then the steps into quotes and lists (see [`Node::get`]).
    pub path: Vec<usize>,
    /// Bytes into the text of the leaf (on a char boundary).
    pub at: usize,
}

/// The shortcut that fired last, so that Backspace right after gives the typed characters back (as in tiptap).
#[derive(Clone, Debug)]
pub(super) struct Fired {
    pub path: Vec<usize>,
    pub typed: String,
    /// The shortcut only put a task box on an item that was there (Backspace takes the box off), not made the block.
    pub box_only: bool,
}

/// The rich editor: typed blocks, and the document as it was loaded to find the operations against.
#[derive(Debug, Default)]
pub struct RichEditor {
    pub(super) blocks: Vec<RichBlock>,
    loaded: Doc,
    pub(super) fired: Option<Fired>,
}

impl RichEditor {
    /// An editor with no document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The typed blocks as they are now (what a frontend draws).
    #[must_use]
    pub fn blocks(&self) -> &[RichBlock] {
        &self.blocks
    }

    /// The node at `path` (the first step is the block).
    #[must_use]
    pub fn node(&self, path: &[usize]) -> Option<&Node> {
        let (&b, rest) = path.split_first()?;
        self.blocks.get(b)?.node.get(rest)
    }

    pub(super) fn node_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let (&b, rest) = path.split_first()?;
        self.blocks.get_mut(b)?.node.get_mut(rest)
    }

    /// The list item a leaf is the text of (the first child of), with the task box state.
    #[must_use]
    pub fn item_of(&self, path: &[usize]) -> Option<&Item> {
        let (&j, up) = path.split_last()?;
        let (&i, list) = up.split_last()?;
        match self.node(list)? {
            Node::List(l) if j == 0 => l.items.get(i),
            _ => None,
        }
    }

    /// Mark the block of `path` as changed.
    pub(super) fn touch(&mut self, path: &[usize]) {
        if let Some(b) = path.first().and_then(|&b| self.blocks.get_mut(b)) {
            b.touch();
        }
    }

    /// The paths of all leaves of the page, in reading order.
    #[must_use]
    pub fn leaves(&self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        for (b, block) in self.blocks.iter().enumerate() {
            out.extend(block.leaves().into_iter().map(|mut p| {
                p.insert(0, b);
                p
            }));
        }
        out
    }

    /// The text of the leaf at `path` (empty for a rule).
    #[must_use]
    pub fn text(&self, path: &[usize]) -> &str {
        self.node(path).and_then(Node::text).unwrap_or("")
    }

    /// The document to be stored. [`TextEditor::document`] is the live form (what was typed is what the source holds, which the
    /// contract wants: a shortcut given back is `## `, not `\## `); stored like that, a paragraph that reads `## x` would be a
    /// heading when loaded again. Here such paragraphs are masked with a backslash (`\## x`), and loading it gives the same
    /// blocks back. A host that stores the page stores this.
    #[must_use]
    pub fn save_form(&self) -> Doc {
        Doc::new(self.blocks.iter().map(RichBlock::to_saved_source).collect())
    }

    /// The cursor for the offset `at` in the Markdown source of `block` (the offset the contract speaks in).
    #[must_use]
    pub fn cursor_at(&self, block: usize, at: usize) -> Option<Cursor> {
        let b = self.blocks.get(block)?;
        let leaves = b.leaves();
        let starts = b.starts();
        let mut k = starts.iter().rposition(|&s| s <= at).unwrap_or(0);
        // On the marker of the next leaf (`- [x] ` before its text) the cursor is at the start of that leaf.
        if let Some(&next) = starts.get(k + 1) {
            let source = b.to_source();
            if source
                .get(..next)
                .is_some_and(|t| at >= t.rfind('\n').map_or(0, |i| i + 1))
            {
                k += 1;
            }
        }
        let mut path = leaves.get(k)?.clone();
        path.insert(0, block);
        let text = self.text(&path);
        let mut at = at
            .saturating_sub(starts.get(k).copied().unwrap_or(0))
            .min(text.len());
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        Some(Cursor { path, at })
    }
}

impl TextEditor for RichEditor {
    fn name(&self) -> &'static str {
        "rich"
    }

    fn input_rules(&self) -> bool {
        true
    }

    fn load(&mut self, doc: &Doc) {
        self.blocks = doc.blocks.iter().map(|s| RichBlock::parse(s)).collect();
        self.loaded = doc.clone();
        self.fired = None;
    }

    fn document(&self) -> Doc {
        Doc::new(self.blocks.iter().map(RichBlock::to_source).collect())
    }

    fn edit(&mut self, edit: &Edit) {
        match edit {
            Edit::Type { block, at, text } => {
                if let Some(mut c) = self.cursor_at(*block, *at) {
                    for ch in text.chars().filter(|&ch| ch != '\n') {
                        c = self.type_char(&c, ch);
                    }
                }
            }
            Edit::Enter { block, at } => {
                if let Some(c) = self.cursor_at(*block, *at) {
                    self.enter(&c);
                }
            }
            Edit::Backspace { block, at } => {
                if let Some(c) = self.cursor_at(*block, *at) {
                    self.backspace(&c);
                }
            }
            Edit::ToggleTask { block, at } => {
                if let Some(c) = self.cursor_at(*block, *at) {
                    self.toggle_task(&c.path);
                }
            }
        }
    }

    fn take_ops(&mut self) -> Vec<Op> {
        let now = self.document();
        let ops = diff(&self.loaded, &now);
        self.loaded = now;
        ops
    }
}
