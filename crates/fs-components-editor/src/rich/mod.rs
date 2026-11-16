//! The `RichEditor`: a typed block model behind the editor contract, with Markdown input rules and cursor keys.
//!
//! Pattern: Strategy (the third editor behind [`crate::TextEditor`]) over an Adapter ([`RichBlock`]) — see [`editor`].
//!
//! The model: a block is a leaf (paragraph, heading, code, rule) or a container (a quote, a list with items), as a tree
//! ([`node`]). The rules ([`rules`]) turn shortcuts into blocks the moment they are typed. The [`Session`] adds what a person at
//! a keyboard has and the contract has not: a cursor that walks over the blocks.

mod back;
mod block;
mod editor;
mod enter;
mod nav;
pub mod node;
mod read;
pub mod rules;
mod session;
mod tree;
mod type_in;
mod write;

pub use block::RichBlock;
pub use editor::{Cursor, RichEditor};
pub use node::{Item, List, Node};
pub use session::{Key, Session};
