//! The neutral description of a text (the model lives in `fs-components-core`, re-exported here unchanged)
//! and the reader that fills it from `CommonMark`.
//!
//! Pattern: none (declares modules and re-exports only) — the paths `fs_components_widgets::text::*` stay as before.

pub mod markdown;

pub use fs_components_core::text::{Align, Block, Document, Inline, ListItem, Mark, MarkKind};
pub use markdown::markdown_blocks;
