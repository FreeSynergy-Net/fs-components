//! The `RichTextEditor` spike: a typed block model behind the editor contract, with Markdown input rules.
//!
//! Pattern: Strategy (the third editor behind [`crate::TextEditor`]) over an Adapter ([`RichBlock`]): a document of Markdown
//! source blocks is read into typed blocks (heading level, task box, code with language, list item, quote, rule); typing
//! changes the typed blocks, the rules ([`rules`]) turn shortcuts into kinds at once, and what goes out is only block
//! operations of the same model. A block nobody touched keeps its source byte for byte.
//!
//! The editor holds no cursor: every edit says where it happens (a byte offset in the Markdown source of the block, as
//! the contract has it). The drawing of the typed blocks is the business of the frontend; this part is the model.

mod block;
mod editor;
pub mod rules;

pub use block::{Kind, RichBlock};
pub use editor::RichEditor;
