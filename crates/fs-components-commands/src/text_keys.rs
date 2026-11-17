//! Which presses belong to a text field and not to the program (the rule of the focus context `TextField`).
//!
//! Pattern: Specification — one predicate, one table of cases in the tests. The rule: a text field keeps what a person types and
//! what every text editor does (typing, copy, paste, cut, select all, undo, redo, moving and deleting, Enter, Tab, Space); the program
//! gets everything else (Ctrl+S, Ctrl+Shift+P, Escape, F-keys, Ctrl+Enter). An Alt letter is typed into the field on some platforms
//! (the spike of M2: `DESIGN.md`, "Alt+letter: state"), and Ctrl+Alt+letter is `AltGr` on Windows and Linux, so both count as text.

use crate::display::Platform;
use crate::keypress::KeyPress;

const EDITING_KEYS: &[&str] = &[
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
];

/// Does the text field own this press (the program must not run a shortcut for it)?
#[must_use]
pub(crate) fn owned_by_text(press: &KeyPress, platform: Platform) -> bool {
    let shortcut = press.shortcut();
    let m = shortcut.modifiers();
    let key = shortcut.key();
    // Ctrl on Linux and Windows, Cmd on macOS: the key every text shortcut is made with.
    let command = if platform == Platform::Mac {
        m.meta
    } else {
        m.control
    };
    if EDITING_KEYS.contains(&key) {
        return !m.alt;
    }
    if matches!(key, "Enter" | "Tab" | "Space") {
        return !command;
    }
    if key.chars().count() != 1 {
        return false;
    }
    // A character key. Without the command key it is typed (also with Shift, Alt, AltGr).
    if !command {
        return true;
    }
    if m.alt && platform != Platform::Mac {
        return true;
    }
    matches!(key, "A" | "C" | "V" | "X" | "Z" | "Y") && !(m.shift && key != "Z")
}
