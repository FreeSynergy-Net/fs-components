//! A Blinc key event as a [`KeyPress`]: the one place that knows Blinc's key codes for shortcuts.
//!
//! Pattern: Adapter (Blinc `KEY_DOWN` code and modifiers to the neutral press of `fs-components-commands`). The codes are the
//! ones the window delivers (`blinc_app` `keyboard.rs`, pin `fs-blinc-3.1`): letters 65-90, digits 48-57, punctuation as the DOM
//! `keyCode`, F10 = 121, the menu key = 93. F1-F9, F11, F12 and Insert have code 0 and no `KEY_DOWN` (Blinc ticket 5aa38f7); the numpad
//! digits equal the digit row (winit reports them as characters). A shortcut with those F keys or Insert cannot run in Blinc yet (`DESIGN.md`, "Shortcuts: state").

use fs_components_commands::{Area, KeyPress, Modifiers, Trigger};

/// The key a Blinc code stands for, as the shortcut text names it; `None` for a code that is no key of a shortcut.
fn key_name(code: u32) -> Option<String> {
    let name = match code {
        8 => "Backspace",
        9 => "Tab",
        13 => "Enter",
        27 => "Escape",
        32 => "Space",
        33 => "PageUp",
        34 => "PageDown",
        35 => "End",
        36 => "Home",
        37 => "ArrowLeft",
        38 => "ArrowUp",
        39 => "ArrowRight",
        40 => "ArrowDown",
        93 => "ContextMenu",
        121 => "F10",
        127 => "Delete",
        48..=57 | 65..=90 => return char::from_u32(code).map(String::from),
        186 => ";",
        187 => "=",
        188 => ",",
        189 => "-",
        190 => ".",
        191 => "/",
        192 => "`",
        219 => "[",
        220 => "\\",
        221 => "]",
        222 => "'",
        _ => return None,
    };
    Some(name.to_owned())
}

/// The press for the key `code` with the modifiers the event carries; `None` for a key that is no shortcut key (a code of 0).
pub(crate) fn press_from_code(
    code: u32,
    (shift, ctrl, alt, meta): (bool, bool, bool, bool),
) -> Option<KeyPress> {
    let modifiers = Modifiers {
        primary: false,
        control: ctrl,
        alt,
        shift,
        meta,
    };
    KeyPress::new(modifiers, &key_name(code)?).ok()
}

/// The context menu a key asks for at the focused element: the menu key (code 93), or Shift+F10 (121). Any other key asks for
/// nothing. The code of the menu key is the one `key_name` knows; F10 alone is no context menu (it is the menu bar in some systems).
pub(crate) fn context_key(code: u32, shift: bool, element: Area) -> Option<Trigger> {
    match (code, shift) {
        (93, _) => Some(Trigger::MenuKey { element }),
        (121, true) => Some(Trigger::ShiftF10 { element }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_become_the_names_a_shortcut_text_uses() {
        let none = (false, false, false, false);
        for (code, shortcut) in [
            (83, "S"),
            (48, "0"),
            (13, "Enter"),
            (127, "Delete"),
            (121, "F10"),
            (93, "ContextMenu"),
            (191, "/"),
        ] {
            let press = press_from_code(code, none).unwrap();
            assert_eq!(press.shortcut().aria(), shortcut, "code {code}");
        }
        assert!(press_from_code(0, none).is_none(), "a code of 0 is no key");
        assert!(
            press_from_code(112, none).is_none(),
            "the window delivers no F1"
        );
    }

    #[test]
    fn modifiers_are_the_keys_themselves() {
        let press = press_from_code(80, (true, true, false, false)).unwrap();
        assert_eq!(press.shortcut().aria(), "Control+Shift+P");
        let press = press_from_code(70, (false, false, true, true)).unwrap();
        assert_eq!(press.shortcut().aria(), "Alt+Meta+F");
    }

    #[test]
    fn the_menu_key_and_shift_f10_ask_for_a_context_menu() {
        let element = Area {
            x: 1.0,
            y: 2.0,
            width: 30.0,
            height: 4.0,
        };
        assert_eq!(
            context_key(93, false, element),
            Some(Trigger::MenuKey { element })
        );
        assert_eq!(
            context_key(93, true, element),
            Some(Trigger::MenuKey { element })
        );
        assert_eq!(
            context_key(121, true, element),
            Some(Trigger::ShiftF10 { element })
        );
        assert_eq!(
            context_key(121, false, element),
            None,
            "F10 alone is no context menu"
        );
        assert_eq!(context_key(83, false, element), None);
    }
}
