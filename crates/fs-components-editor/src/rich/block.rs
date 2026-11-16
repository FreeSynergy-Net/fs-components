//! One top-level block of the rich editor: the typed tree and the source it came from while nobody has changed it.
//!
//! Pattern: Memento (the source as it was read) over an Adapter ([`super::read`], [`super::write`]) — a block nobody touched is
//! given back byte for byte; a touched block is written in the canonical form of its kinds.

use super::node::Node;
use super::{read, write};

/// A block of the document as the rich editor holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichBlock {
    /// What the block is.
    pub node: Node,
    /// The Markdown source the block was read from; `Some` only while the block is unchanged.
    source: Option<String>,
    /// Where the text of each leaf begins in the source it was read from (only while the block is unchanged).
    read_starts: Vec<usize>,
}

impl RichBlock {
    /// A block of `node` with no source (written in the canonical form).
    #[must_use]
    pub fn new(node: Node) -> Self {
        Self {
            node,
            source: None,
            read_starts: Vec::new(),
        }
    }

    /// Read one block of Markdown source (without the blank lines around it).
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let (node, starts) = read::read(source);
        Self {
            node,
            source: Some(source.to_owned()),
            read_starts: starts,
        }
    }

    /// The Markdown source of the block: the original while unchanged, else the canonical form.
    #[must_use]
    pub fn to_source(&self) -> String {
        self.source
            .clone()
            .unwrap_or_else(|| write::write(&self.node).0)
    }

    /// The block changed: its source is no longer the truth, it is written from the tree from now on.
    pub fn touch(&mut self) {
        self.source = None;
    }

    /// Where the text of each leaf begins in [`RichBlock::to_source`], in reading order.
    #[must_use]
    pub fn starts(&self) -> Vec<usize> {
        match self.source {
            Some(_) => self.read_starts.clone(),
            None => write::write(&self.node).1,
        }
    }

    /// The paths of the leaves of this block (below the block itself), in reading order.
    #[must_use]
    pub fn leaves(&self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        self.node.leaves(&mut Vec::new(), &mut out);
        out
    }
}
