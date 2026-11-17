//! A keyboard shortcut and its stored form: W3C `aria-keyshortcuts` plus `Primary`.
//!
//! Pattern: Value object with a parser (text to value) and its inverse (`aria()`); the pair is a round trip. `Primary` is Ctrl
//! on Linux and Windows and Cmd on macOS (GTK `<Primary>`), so one stored shortcut fits every platform.

use std::fmt;

use crate::display::Platform;

/// The modifier keys of a shortcut: five independent keys, so five flags are the plain shape.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Ctrl, or Cmd on macOS (not part of `aria-keyshortcuts`).
    pub primary: bool,
    /// The Control key itself.
    pub control: bool,
    /// Alt (Option on macOS).
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Meta (Super, Windows key, Cmd on macOS).
    pub meta: bool,
}

impl Modifiers {
    /// `Primary` replaced by what it is on `platform`; two shortcuts that differ only in the spelling of it are equal then.
    #[must_use]
    pub fn resolved(self, platform: Platform) -> Self {
        let mut out = Self {
            primary: false,
            ..self
        };
        if self.primary {
            if platform == Platform::Mac {
                out.meta = true;
            } else {
                out.control = true;
            }
        }
        out
    }
}

/// Why a text is no shortcut.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShortcutError {
    /// Nothing there.
    Empty,
    /// A modifier name twice (`Control+Control+S`).
    DuplicateModifier(String),
    /// Only modifiers, no key (`Control+Shift`).
    NoKey,
    /// More than one key (`Control+S+T`).
    TwoKeys(String),
    /// A word that is neither a modifier nor a key name.
    UnknownKey(String),
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty shortcut"),
            Self::DuplicateModifier(m) => write!(f, "modifier {m:?} twice"),
            Self::NoKey => write!(f, "shortcut without a key"),
            Self::TwoKeys(k) => write!(f, "second key {k:?}"),
            Self::UnknownKey(k) => write!(f, "unknown key {k:?}"),
        }
    }
}

impl std::error::Error for ShortcutError {}

/// The key names besides a single character (the values of `KeyboardEvent.key`).
pub(crate) const NAMED_KEYS: &[&str] = &[
    "Enter",
    "Escape",
    "Tab",
    "Space",
    "Backspace",
    "Delete",
    "Insert",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ContextMenu",
];

/// One shortcut: modifiers and one key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    modifiers: Modifiers,
    key: String,
}

impl Shortcut {
    /// The modifiers.
    #[must_use]
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// The key: one upper-case character, or a name (`Enter`, `F5`, `ArrowUp`).
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Parse one shortcut: `Control+Shift+P`, `Primary+S`, `F1`, `Alt+ArrowDown`, `Primary++` (the plus key).
    ///
    /// # Errors
    /// [`ShortcutError`] if the text is empty, has a modifier twice, no key, two keys or an unknown name.
    pub fn parse(text: &str) -> Result<Self, ShortcutError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ShortcutError::Empty);
        }
        // A trailing `+` is the plus key: `Primary++`, or `+` alone. Before it stands nothing or modifiers ending in `+`.
        let (head, mut key) = match text.strip_suffix('+') {
            Some("") => ("", Some("+".to_owned())),
            Some(rest) => match rest.strip_suffix('+') {
                Some(mods) => (mods, Some("+".to_owned())),
                None => return Err(ShortcutError::NoKey),
            },
            None => (text, None),
        };
        let plus_key = key.is_some();
        let mut modifiers = Modifiers::default();
        for part in head.split('+').filter(|p| !p.is_empty() || !plus_key) {
            if let Some(slot) = modifier_slot(&mut modifiers, part) {
                if *slot {
                    return Err(ShortcutError::DuplicateModifier(part.to_owned()));
                }
                *slot = true;
            } else if key.is_some() {
                return Err(ShortcutError::TwoKeys(part.to_owned()));
            } else {
                key = Some(normal_key(part)?);
            }
        }
        key.map(|key| Self { modifiers, key })
            .ok_or(ShortcutError::NoKey)
    }

    /// Parse several shortcuts separated by spaces (`Control+S Meta+S`), as `aria-keyshortcuts` writes them.
    ///
    /// # Errors
    /// The first error of [`Shortcut::parse`].
    pub fn parse_list(text: &str) -> Result<Vec<Self>, ShortcutError> {
        text.split_whitespace().map(Self::parse).collect()
    }

    /// The stored form, modifiers in a fixed order: `Primary+Control+Alt+Shift+Meta+Key`. `parse` of it is this shortcut.
    #[must_use]
    pub fn aria(&self) -> String {
        let m = self.modifiers;
        let mut out = String::new();
        for (on, name) in [
            (m.primary, "Primary"),
            (m.control, "Control"),
            (m.alt, "Alt"),
            (m.shift, "Shift"),
            (m.meta, "Meta"),
        ] {
            if on {
                out.push_str(name);
                out.push('+');
            }
        }
        out.push_str(&self.key);
        out
    }

    /// Is this the same key combination as `other` on `platform` (`Primary+S` and `Control+S` are on Linux)?
    #[must_use]
    pub fn same_as(&self, other: &Self, platform: Platform) -> bool {
        self.key == other.key
            && self.modifiers.resolved(platform) == other.modifiers.resolved(platform)
    }
}

fn modifier_slot<'a>(m: &'a mut Modifiers, name: &str) -> Option<&'a mut bool> {
    match name.to_ascii_lowercase().as_str() {
        "primary" => Some(&mut m.primary),
        "control" | "ctrl" => Some(&mut m.control),
        "alt" => Some(&mut m.alt),
        "shift" => Some(&mut m.shift),
        "meta" => Some(&mut m.meta),
        _ => None,
    }
}

fn normal_key(part: &str) -> Result<String, ShortcutError> {
    let mut chars = part.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return Ok(c.to_uppercase().collect());
    }
    if let Some(name) = NAMED_KEYS.iter().find(|n| n.eq_ignore_ascii_case(part)) {
        return Ok((*name).to_owned());
    }
    if let Some(number) = part
        .strip_prefix(['F', 'f'])
        .and_then(|n| n.parse::<u8>().ok())
        && (1..=24).contains(&number)
    {
        return Ok(format!("F{number}"));
    }
    Err(ShortcutError::UnknownKey(part.to_owned()))
}
