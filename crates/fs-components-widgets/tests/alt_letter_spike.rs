//! Spike b7 (M2): does Alt+letter also type into a text field? Probe at the Blinc pin `fs-blinc-3.1` (cf925af).
//!
//! The window (`blinc_app` `keyboard.rs`) sends, for a press with a character, `TEXT_INPUT` with the modifiers (only Ctrl without
//! `AltGr` and Meta are left out), then `KEY_DOWN` with the key code. Both are broadcast to every handler. This test feeds that same
//! sequence into a real text field and a global key hook. What it cannot show: whether the platform delivers a character for Alt+F
//! at all (winit text, platform dependent; on X11 and Wayland it is `f`).
//!
//! Tests: Input probe over the built name dialog (a Blinc text input).

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use common::{layout, serial};
use fs_components_widgets::name_dialog::{Labels, NameDialog, Purpose};

/// What the hook saw: (key code, Alt held).
type Seen = Arc<Mutex<Vec<(u32, bool)>>>;

fn field() -> (NameDialog, blinc_layout::prelude::RenderTree, Seen) {
    let seen: Seen = Arc::default();
    let sink = Arc::clone(&seen);
    let d = NameDialog::new(&Labels::default(), Purpose::NewPage, "").id("alt");
    // The hook the app shell puts on its root element: every key press, whoever has the keyboard.
    let root = div()
        .on_key_down(move |ctx| sink.lock().unwrap().push((ctx.key_code, ctx.alt)))
        .child(d.build());
    let tree = layout(root, 480.0, 260.0);
    d.focus();
    blinc_layout::widgets::text_input::process_pending_input_focus();
    (d, tree, seen)
}

/// What the window does for a press with a character: `TEXT_INPUT`, then `KEY_DOWN`.
fn press(
    tree: &mut blinc_layout::prelude::RenderTree,
    ch: char,
    code: u32,
    (shift, ctrl, alt, meta): (bool, bool, bool, bool),
) {
    if !ctrl && !meta {
        tree.broadcast_text_input_event(ch, shift, ctrl, alt, meta);
    }
    tree.broadcast_key_event(KEY_DOWN, code, shift, ctrl, alt, meta);
}

#[test]
fn alt_letter_is_typed_into_the_field_and_reaches_the_hook_after_it() {
    let _g = serial();
    let (d, mut tree, seen) = field();
    press(&mut tree, 'f', 70, (false, false, true, false));
    assert_eq!(
        d.text(),
        "f",
        "CONFIRMED at the pin: Alt+F types `f` into the field (the field checks Ctrl and Meta, never Alt)"
    );
    assert_eq!(
        *seen.lock().unwrap(),
        [(70, true)],
        "the hook sees Alt+F, but only after the character is in"
    );
}

#[test]
fn ctrl_and_meta_letters_are_not_typed() {
    let _g = serial();
    let (d, mut tree, seen) = field();
    press(&mut tree, 'f', 70, (false, true, false, false));
    press(&mut tree, 'f', 70, (false, false, false, true));
    assert_eq!(
        d.text(),
        "",
        "the window sends no TEXT_INPUT for Ctrl or Meta, and the field ignores them if it came"
    );
    tree.broadcast_text_input_event('f', false, true, false, false);
    tree.broadcast_text_input_event('f', false, false, false, true);
    assert_eq!(d.text(), "");
    assert_eq!(seen.lock().unwrap().len(), 2);
}

#[test]
fn a_plain_letter_is_typed() {
    let _g = serial();
    let (d, mut tree, _) = field();
    press(&mut tree, 'f', 70, (false, false, false, false));
    assert_eq!(d.text(), "f");
}
