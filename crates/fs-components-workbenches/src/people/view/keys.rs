//! The keyboard of an element with a context menu: the menu key, Shift+F10, and the shortcuts of the keymap.
//!
//! Pattern: Adapter — Blinc's key events to the model's key press. Like `bind_keys` of the widgets it acts only on the element that
//! has the keyboard (Blinc may broadcast a key to every handler), but it reads the rectangle of the element too, because a menu
//! opened by the keyboard hangs at the element, never at the pointer.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::Div;
use blinc_layout::event_router::keyboard_focused_node;
use fs_components_commands::{Area, Trigger};
use fs_components_widgets::keys::{Key, KeyEvent};

use crate::people::keypress::asks_for_menu;

/// What the element does with a key it has the keyboard for.
pub(in crate::people) struct KeyActs {
    /// The menu key or Shift+F10: where the menu goes.
    pub menu: Arc<dyn Fn(Trigger) + Send + Sync>,
    /// Any other key, as the model's press; the window asks the keymap.
    pub shortcut: Arc<dyn Fn(&KeyEvent) + Send + Sync>,
    /// Enter or Space on the element.
    pub activate: Arc<dyn Fn() + Send + Sync>,
}

/// Make `row` a tab stop that hears the keyboard while it has it.
pub(in crate::people) fn with_keys(row: Div, acts: KeyActs) -> Div {
    let focused = Arc::new(AtomicBool::new(false));
    let (on, off) = (Arc::clone(&focused), Arc::clone(&focused));
    let KeyActs {
        menu,
        shortcut,
        activate,
    } = acts;
    row.attr("tabindex", "0")
        .on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| off.store(false, Ordering::Relaxed))
        .on_click(move |ctx| {
            // Enter or Space arrive as a click on the focused element; a mouse press clears this mode.
            if keyboard_focused_node() == Some(ctx.node_id) {
                activate();
            }
        })
        .on_key_down(move |ctx| {
            let mine = match keyboard_focused_node() {
                Some(node) => node == ctx.node_id,
                None => focused.load(Ordering::Relaxed),
            };
            if !mine || blinc_layout::widgets::text_input::has_focused_text_input() {
                return;
            }
            let mut event = KeyEvent::new(Key::from_code(ctx.key_code));
            event.shift = ctx.shift;
            event.ctrl = ctx.ctrl;
            event.alt = ctx.alt;
            if asks_for_menu(&event) {
                let element = Area {
                    x: ctx.bounds_x,
                    y: ctx.bounds_y,
                    width: ctx.bounds_width,
                    height: ctx.bounds_height,
                };
                menu(if event.shift {
                    Trigger::ShiftF10 { element }
                } else {
                    Trigger::MenuKey { element }
                });
            } else {
                shortcut(&event);
            }
        })
}
