//! A key of the window as the command model's key press, and the keys that ask for a context menu.
//!
//! Pattern: Adapter — Blinc's key codes (the DOM-style numbers the window delivers) to the model's [`KeyPress`]; the keymap
//! (`PeopleState::keymap`) decides what a press means, nothing here does.

use fs_components_commands::{KeyPress, Modifiers};
use fs_components_widgets::keys::{Key, KeyEvent};

/// The key press of the model for a key of the window; `None` for a key the model has no name for (modifiers alone, unknown codes).
pub fn press_from(event: &KeyEvent) -> Option<KeyPress> {
    let name: String = match event.key {
        Key::Enter | Key::Activate => "Enter".into(),
        Key::Space => "Space".into(),
        Key::Escape => "Escape".into(),
        Key::Up => "ArrowUp".into(),
        Key::Down => "ArrowDown".into(),
        Key::Left => "ArrowLeft".into(),
        Key::Right => "ArrowRight".into(),
        Key::Home => "Home".into(),
        Key::End => "End".into(),
        Key::PageUp => "PageUp".into(),
        Key::PageDown => "PageDown".into(),
        Key::Other(code) => other_name(code)?,
        _ => return None,
    };
    let modifiers = Modifiers {
        control: event.ctrl,
        shift: event.shift,
        alt: event.alt,
        ..Modifiers::default()
    };
    KeyPress::new(modifiers, &name).ok()
}

/// The name of a key by the platform's code: letters, digits, Delete, the F keys.
fn other_name(code: u32) -> Option<String> {
    match code {
        65..=90 | 48..=57 => char::from_u32(code).map(String::from),
        46 => Some("Delete".into()),
        112..=123 => Some(format!("F{}", code - 111)),
        _ => None,
    }
}

/// Is this the key that asks for a context menu: the menu key, or Shift+F10?
pub fn asks_for_menu(event: &KeyEvent) -> bool {
    matches!(event.key, Key::Other(93)) || (event.shift && matches!(event.key, Key::Other(121)))
}
