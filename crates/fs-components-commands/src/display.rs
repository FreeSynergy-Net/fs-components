//! How a shortcut is shown: per platform (Ctrl or Cmd) and per language (Strg, Umschalt).
//!
//! Pattern: Strategy — the platform chooses the style (words joined by `+`, or the symbols of macOS), the `KeyNames` hold the words
//! of the language. The names come from the program's MF2 texts (ADR 0057); this crate holds no language of its own besides two
//! ready-made tables for tests and for programs without texts.

use std::collections::BTreeMap;

use crate::shortcut::Shortcut;

/// The platform a shortcut is shown for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// Linux and the BSDs.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    Mac,
}

impl Platform {
    /// The platform this program was built for.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// The words of the modifier keys in one language, and the names of keys that are not written as themselves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyNames {
    control: String,
    alt: String,
    shift: String,
    meta: String,
    keys: BTreeMap<String, String>,
}

impl KeyNames {
    /// The four modifier words.
    pub fn new(
        control: impl Into<String>,
        alt: impl Into<String>,
        shift: impl Into<String>,
        meta: impl Into<String>,
    ) -> Self {
        Self {
            control: control.into(),
            alt: alt.into(),
            shift: shift.into(),
            meta: meta.into(),
            keys: BTreeMap::new(),
        }
    }

    /// The word of a named key in this language (`Delete` as `Entf`).
    #[must_use]
    pub fn key(mut self, name: &str, word: impl Into<String>) -> Self {
        self.keys.insert(name.to_owned(), word.into());
        self
    }

    /// English: Ctrl, Alt, Shift, Super.
    #[must_use]
    pub fn english() -> Self {
        Self::new("Ctrl", "Alt", "Shift", "Super")
    }

    /// German: Strg, Alt, Umschalt, Super; Entf for Delete.
    #[must_use]
    pub fn german() -> Self {
        Self::new("Strg", "Alt", "Umschalt", "Super")
            .key("Delete", "Entf")
            .key("Backspace", "Rück")
            .key("PageUp", "Bild\u{2191}")
            .key("PageDown", "Bild\u{2193}")
    }

    fn named(&self, key: &str) -> String {
        if let Some(word) = self.keys.get(key) {
            return word.clone();
        }
        match key {
            "ArrowUp" => "\u{2191}".to_owned(),
            "ArrowDown" => "\u{2193}".to_owned(),
            "ArrowLeft" => "\u{2190}".to_owned(),
            "ArrowRight" => "\u{2192}".to_owned(),
            "Escape" => "Esc".to_owned(),
            "Delete" => "Del".to_owned(),
            "Backspace" => "Backspace".to_owned(),
            other => other.to_owned(),
        }
    }
}

impl Shortcut {
    /// The text a menu shows at the right of its line.
    ///
    /// Linux and Windows: the words joined by `+` (`Strg+Umschalt+P`). macOS: the symbols in the order of its menus, without
    /// a separator (`⌃⌥⇧⌘P`); `Primary` is Cmd there and Ctrl elsewhere.
    #[must_use]
    pub fn display(&self, platform: Platform, names: &KeyNames) -> String {
        let m = self.modifiers().resolved(platform);
        let key = if self.key().chars().count() == 1 {
            self.key().to_owned()
        } else {
            names.named(self.key())
        };
        if platform == Platform::Mac {
            let mut out = String::new();
            for (on, symbol) in [
                (m.control, "\u{2303}"),
                (m.alt, "\u{2325}"),
                (m.shift, "\u{21e7}"),
                (m.meta, "\u{2318}"),
            ] {
                if on {
                    out.push_str(symbol);
                }
            }
            out.push_str(&key);
            out
        } else {
            let mut parts: Vec<&str> = Vec::new();
            for (on, word) in [
                (m.control, names.control.as_str()),
                (m.alt, names.alt.as_str()),
                (m.shift, names.shift.as_str()),
                (m.meta, names.meta.as_str()),
            ] {
                if on {
                    parts.push(word);
                }
            }
            parts.push(&key);
            parts.join("+")
        }
    }
}
