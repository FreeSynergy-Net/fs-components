//! The trait every editor building block stands behind.
//!
//! Pattern: Strategy — the program picks a building block; all of them take a [`Doc`] and give back [`Op`]s.
//!
//! Contract versions: 1 had `Type` and `Enter`. **2** adds `Backspace` and `ToggleTask` (every editor must do them, the
//! collection checks it) and the capability [`TextEditor::input_rules`] (an editor that turns shortcuts typed into a block into
//! typed blocks says so, and the collection then checks the shortcuts). [`CONTRACT_VERSION`] says which one this is.

/// The version of the editor contract (see the head of this file).
pub const CONTRACT_VERSION: u32 = 2;

use crate::{Doc, Op};

/// A change a person makes, as the keyboard makes it. The conformance tests drive every editor with these, so they say
/// what happens to the text, not how a particular editor does it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Type `text` into `block`, `at` bytes into it.
    Type {
        /// Which block.
        block: usize,
        /// Byte offset in the block.
        at: usize,
        /// What is typed (no line break).
        text: String,
    },
    /// Press Enter in `block`, `at` bytes into it.
    Enter {
        /// Which block.
        block: usize,
        /// Byte offset in the block.
        at: usize,
    },
    /// Press Backspace in `block`, `at` bytes into it (version 2): the character before `at` goes; at the start of a block
    /// the block is joined to the one above (its text is appended there). An editor with typed blocks may first turn a typed
    /// block (a heading, a list item) into a paragraph, as a person expects; the fixtures of the collection use paragraphs.
    Backspace {
        /// Which block.
        block: usize,
        /// Byte offset in the block.
        at: usize,
    },
    /// Tick or untick the task item on the line of `block` that `at` is on (version 2); nothing when that line is no task item.
    ToggleTask {
        /// Which block.
        block: usize,
        /// Byte offset in the block, anywhere on the line of the item.
        at: usize,
    },
}

/// An editor building block: a document goes in, only change operations of the same model come out.
///
/// Rules (the conformance tests check them): a document that is loaded and read back without an edit is the same
/// document and [`TextEditor::take_ops`] gives nothing; after edits, applying `take_ops` to the loaded document gives
/// exactly [`TextEditor::document`]; an editor changes data in no other way.
pub trait TextEditor {
    /// The fixed name of the building block (`source`, `block`, `rich`); also the base of its style names.
    fn name(&self) -> &'static str;

    /// Start editing `doc`.
    fn load(&mut self, doc: &Doc);

    /// The document as the editor holds it now.
    fn document(&self) -> Doc;

    /// Does typing a Markdown shortcut (`## `, `- `, `[] ` ...) make a typed block at once? A source editor says no (it shows what
    /// is typed); the collection checks the shortcuts only of an editor that says yes.
    fn input_rules(&self) -> bool {
        false
    }

    /// Do one edit, as the keyboard would.
    fn edit(&mut self, edit: &Edit);

    /// The operations since `load` (or since the last call) that turn the document as it was then into the one now.
    fn take_ops(&mut self) -> Vec<Op>;
}
