//! What a person at a keyboard has: a cursor that walks over the blocks, and keys that act at it.
//!
//! Pattern: Facade over [`RichEditor`] — one small type with the cursor and the one entry for keys ([`Session::key`]) and clicks
//! ([`Session::click_task`], [`Session::place`]); a frontend draws [`Session::editor`] and the cursor, nothing more. The contract
//! of the editor building blocks has no cursor (ADR 0084 leaves that open); the session is where it lives.

use super::block::RichBlock;
use super::editor::{Cursor, RichEditor};
use super::node::Node;
use crate::{Doc, TextEditor};

/// A key of the keyboard as the writing surface hears it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A printable character (after the input method).
    Char(char),
    /// Enter.
    Enter,
    /// Backspace.
    Backspace,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Start of the line.
    Home,
    /// End of the line.
    End,
}

/// An editor with a cursor.
#[derive(Debug, Default)]
pub struct Session {
    editor: RichEditor,
    cursor: Cursor,
    /// The column (in characters) the person is after while going up and down; none after any other key.
    want: Option<usize>,
}

impl Session {
    /// A session over `doc`, the cursor at the start of the first leaf.
    #[must_use]
    pub fn new(doc: &Doc) -> Self {
        let mut s = Self::default();
        s.editor.load(doc);
        s.ensure_leaf();
        s.cursor = Cursor {
            path: s.editor.leaves().first().cloned().unwrap_or_default(),
            at: 0,
        };
        s
    }

    /// The editor (what a frontend draws, and the contract: `document`, `take_ops`).
    #[must_use]
    pub fn editor(&self) -> &RichEditor {
        &self.editor
    }

    /// The cursor.
    #[must_use]
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// The document as it is now.
    #[must_use]
    pub fn document(&self) -> Doc {
        self.editor.document()
    }

    /// The document to be stored (see [`RichEditor::save_form`]).
    #[must_use]
    pub fn save_form(&self) -> Doc {
        self.editor.save_form()
    }

    /// The operations since the session began (or since the last call).
    pub fn take_ops(&mut self) -> Vec<crate::Op> {
        self.editor.take_ops()
    }

    /// A page with no block gets an empty paragraph to write in.
    fn ensure_leaf(&mut self) {
        if self.editor.blocks.is_empty() {
            self.editor
                .blocks
                .push(RichBlock::new(Node::Para(String::new())));
        }
    }

    /// Put the cursor in the leaf at `path`, `at` bytes into its text (clamped to the text and to a character).
    pub fn place(&mut self, path: &[usize], at: usize) {
        if self.editor.node(path).is_some_and(Node::is_leaf) {
            let text = self.editor.text(path);
            let mut at = at.min(text.len());
            while !text.is_char_boundary(at) {
                at -= 1;
            }
            self.cursor = Cursor {
                path: path.to_vec(),
                at,
            };
            self.want = None;
            self.editor.fired = None;
        }
    }

    /// A click on the box of the task item whose text is the leaf at `path`.
    pub fn click_task(&mut self, path: &[usize]) {
        self.editor.toggle_task(path);
        self.editor.fired = None;
    }

    /// One key.
    pub fn key(&mut self, key: Key) {
        self.ensure_leaf();
        if self.editor.node(&self.cursor.path).is_none() {
            self.cursor = Cursor {
                path: self.editor.leaves().first().cloned().unwrap_or_default(),
                at: 0,
            };
        }
        let keep = matches!(key, Key::Up | Key::Down);
        // A shortcut is given back only by the Backspace straight after it: any key of the cursor walking ends that.
        if matches!(
            key,
            Key::Left | Key::Right | Key::Up | Key::Down | Key::Home | Key::End
        ) {
            self.editor.fired = None;
        }
        let c = self.cursor.clone();
        self.cursor = match key {
            Key::Char(ch) if !ch.is_control() => self.editor.type_char(&c, ch),
            Key::Char(_) => c,
            Key::Enter => self.editor.enter(&c),
            Key::Backspace => self.editor.backspace(&c),
            Key::Left => super::nav::left(&self.editor, &c),
            Key::Right => super::nav::right(&self.editor, &c),
            Key::Home => super::nav::home(&self.editor, &c),
            Key::End => super::nav::end(&self.editor, &c),
            Key::Up | Key::Down => {
                let (moved, col) =
                    super::nav::vertical(&self.editor, &c, key == Key::Down, self.want);
                self.want = Some(col);
                moved
            }
        };
        if !keep {
            self.want = None;
        }
    }
}
