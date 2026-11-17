//! A key press as a frontend reports it, and where the keyboard is.
//!
//! Pattern: Value object + Adapter target. A frontend (Blinc, a frontend in its own process) turns its keyboard event into a
//! [`KeyPress`]; the dispatcher ([`crate::Keymap`]) never sees a platform type. The key is compared as a key, not as a place on the
//! keyboard: `Z` is `Z` on a QWERTZ keyboard as well.

use crate::shortcut::{Modifiers, Shortcut, ShortcutError};

/// Where the keyboard is when a key is pressed. Decides which shortcuts count: in a text field the text keeps its own.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FocusContext {
    /// Nowhere in particular: only shortcuts for the whole window count.
    #[default]
    Global,
    /// A list or table has the keyboard (Delete, F2 and Ctrl+A belong to the rows).
    List,
    /// A tree has the keyboard.
    Tree,
    /// A text field has the keyboard: typing, Ctrl+C/V/X/A/Z, arrows and the Alt letters belong to the text.
    TextField,
}

/// One key press: the key with the modifiers held, and whether an input method is composing text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPress {
    shortcut: Shortcut,
    composing: bool,
}

impl KeyPress {
    /// The press of `key` (a character or a key name as in [`Shortcut::parse`]: `s`, `Enter`, `F5`) with `modifiers`. Set
    /// `primary` of the modifiers never: Ctrl and Cmd are the keys themselves here (`control`, `meta`).
    ///
    /// # Errors
    /// [`ShortcutError`] for an empty key or a word that is no key name.
    pub fn new(modifiers: Modifiers, key: &str) -> Result<Self, ShortcutError> {
        Ok(Self {
            shortcut: Shortcut::new(
                Modifiers {
                    primary: false,
                    ..modifiers
                },
                key,
            )?,
            composing: false,
        })
    }

    /// The same press while an input method (IME) composes: it is never a shortcut, the composition owns every key.
    #[must_use]
    pub fn composing(mut self, composing: bool) -> Self {
        self.composing = composing;
        self
    }

    /// The key and the modifiers as a shortcut.
    #[must_use]
    pub fn shortcut(&self) -> &Shortcut {
        &self.shortcut
    }

    /// Is an input method composing?
    #[must_use]
    pub fn is_composing(&self) -> bool {
        self.composing
    }
}
