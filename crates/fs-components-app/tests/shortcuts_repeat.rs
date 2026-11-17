//! A held key runs its command once: Blinc's key event has no repeat flag (pin `fs-blinc-3.1`), so the shell remembers the held key
//! until its release. Tests: two `KEY_DOWN` without `KEY_UP` are one press; the real shell and Blinc's broadcast.

#[path = "menu_common/mod.rs"]
mod menu_common;
#[path = "menu_model_common/mod.rs"]
mod menu_model_common;
#[path = "shortcuts_common/mod.rs"]
mod shortcuts_common;

use shortcuts_common::{C, N, key, log, program, release};

#[test]
fn a_held_key_runs_the_command_once_not_once_per_repeat() {
    // Blinc's key event has no repeat flag (pin cf925af): two KEY_DOWN without KEY_UP are one press held down.
    let (mut h, seen) = program(None);
    key(&mut h, N, true);
    key(&mut h, N, true);
    key(&mut h, N, true);
    assert_eq!(log(&seen), ["document-new"], "repeat is ignored");
    release(&mut h, N, true);
    key(&mut h, N, true);
    assert_eq!(
        log(&seen),
        ["document-new", "document-new"],
        "after the release it runs again"
    );
}

#[test]
fn another_key_in_between_ends_the_hold_and_a_release_of_another_key_does_not() {
    let (mut h, seen) = program(None);
    key(&mut h, N, true);
    key(&mut h, C, true);
    key(&mut h, N, true);
    assert_eq!(log(&seen), ["document-new", "edit-copy", "document-new"]);
    // N is held again; the release of C (a lost or late release) must not make N's repeat run.
    release(&mut h, C, true);
    key(&mut h, N, true);
    assert_eq!(log(&seen).len(), 3);
}
