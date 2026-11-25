//! Keys, as the widgets understand them.
//!
//! Pattern: Adapter — [`Key`] is our neutral name for a key (the platform's key codes
//! stay inside this file). A widget that wants keys binds them to its own element with
//! [`bind_keys`], which lets an event through only when this element is the one that has the
//! keyboard: Blinc may hand a key press to every handler in the window (broadcast), and two
//! widgets must never both act on one press.
//!
//! Real key order of a Blinc window: Tab, Enter and Space go to the window's navigation first; Enter and Space become a
//! click on the focused element (no key-down follows, and the click does not say which of the two it was). Everything else is
//! broadcast as a key event. The RELEASE of Enter and Space is broadcast as a key-up with the key code, so `bind_keys` arms on
//! that click (only when it comes from the keyboard: a mouse press clears the keyboard mode) and reports the press as
//! [`Key::Enter`] or [`Key::Space`] when the key comes up. The two are told apart; nothing reports [`Key::Activate`] any more.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::Div;
use blinc_layout::event_router::keyboard_focused_node;

/// A key a widget may act on.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Enter / Return.
    Enter,
    /// The space bar.
    Space,
    /// Escape.
    Escape,
    /// Delete.
    Delete,
    /// Backspace.
    Backspace,
    /// Enter or Space, not told apart. `bind_keys` no longer reports it (it reports [`Key::Enter`] or [`Key::Space`] on the
    /// release of the key); it stays for a frontend that cannot tell the two apart and for the bindings that name it.
    Activate,
    /// Any other key, by the platform's code (a widget ignores it).
    Other(u32),
}

impl Key {
    /// The key for a platform key code (the DOM-style codes Blinc delivers).
    pub fn from_code(code: u32) -> Self {
        match code {
            8 => Self::Backspace,
            13 => Self::Enter,
            27 => Self::Escape,
            32 => Self::Space,
            33 => Self::PageUp,
            34 => Self::PageDown,
            35 => Self::End,
            36 => Self::Home,
            37 => Self::Left,
            38 => Self::Up,
            39 => Self::Right,
            40 => Self::Down,
            46 => Self::Delete,
            other => Self::Other(other),
        }
    }
}

/// A key press with the modifiers held.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key.
    pub key: Key,
    /// Shift held.
    pub shift: bool,
    /// Control held.
    pub ctrl: bool,
    /// Alt held.
    pub alt: bool,
}

impl KeyEvent {
    /// A press of `key` without modifiers.
    pub fn new(key: Key) -> Self {
        Self {
            key,
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
}

/// What a key handler did with the press (the caller may use it to stop it travelling on).
pub type Handled = bool;

/// Make `div` a tab stop whose key handler runs only while `div` itself has the keyboard.
///
/// `handler` gets the press and says whether it used it.
pub fn bind_keys(div: Div, handler: Arc<dyn Fn(KeyEvent) -> Handled + Send + Sync>) -> Div {
    let focused = Arc::new(AtomicBool::new(false));
    let (on, off) = (Arc::clone(&focused), Arc::clone(&focused));
    // Enter or Space, turned into a click by the window, waits here for its release (which says which key it was).
    let armed = Arc::new(AtomicBool::new(false));
    let (arm, disarm, release) = (Arc::clone(&armed), Arc::clone(&armed), armed);
    let on_release = Arc::clone(&handler);
    div.attr("tabindex", "0")
        .on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| {
            off.store(false, Ordering::Relaxed);
            disarm.store(false, Ordering::Relaxed);
        })
        .on_click(move |ctx| {
            // A mouse click clears the keyboard mode, so only Enter or Space arms.
            if keyboard_focused_node() == Some(ctx.node_id) {
                arm.store(true, Ordering::Relaxed);
            }
        })
        .on_key_up(move |ctx| {
            let key = Key::from_code(ctx.key_code);
            if matches!(key, Key::Enter | Key::Space)
                && keyboard_focused_node() == Some(ctx.node_id)
                && release.swap(false, Ordering::Relaxed)
            {
                on_release(KeyEvent::new(key));
            }
        })
        .on_key_down(move |ctx| {
            let mine = match keyboard_focused_node() {
                Some(node) => node == ctx.node_id,
                None => focused.load(Ordering::Relaxed),
            };
            if mine && !blinc_layout::widgets::text_input::has_focused_text_input() {
                let mut event = KeyEvent::new(Key::from_code(ctx.key_code));
                event.shift = ctx.shift;
                event.ctrl = ctx.ctrl;
                event.alt = ctx.alt;
                handler(event);
            }
        })
}
