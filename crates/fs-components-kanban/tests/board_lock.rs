//! The serial lock of the board tests never hangs: a second `Rig` in one test, or another test that holds the lock for too long,
//! turns red with a plain text ([TEST-TIMEOUT]) instead of waiting for ever.
//!
//! Mutants: no check for a second holder in the same test -> `a_second_rig_in_one_test_is_red_not_a_hang`; the wait has no limit
//! or the text is gone -> `a_lock_held_too_long_is_red_with_a_text`; the mark is not cleared on drop ->
//! `after_drop_the_next_rig_may_come`.
//!
//! Pattern: none; tests (each wait is guarded by a watchdog: a hang is a red test within seconds).
#![allow(clippy::pedantic)]

mod board_common;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use board_common::{Rig, sample, serial, serial_within};
use fs_components_kanban::{BoardBench, Labels};

fn rig() -> Rig {
    let bench = BoardBench::new(Arc::new(sample()), Labels::default()).id("B");
    Rig::new(bench, 800.0, 500.0)
}

fn said(e: &(dyn std::any::Any + Send)) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_default()
}

/// Runs `body` in its own thread (its own "test": its own mark); a hang is a panic of this test after `limit`.
fn guarded<T: Send + 'static>(limit: Duration, body: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(catch_unwind(AssertUnwindSafe(body)));
    });
    match rx.recv_timeout(limit) {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => panic!("the body panicked: {}", said(&*e)),
        Err(_) => panic!("HANG: the body did not end within {limit:?}"),
    }
}

#[test]
fn a_second_rig_in_one_test_is_red_not_a_hang() {
    let message = guarded(Duration::from_secs(20), || {
        let _first = rig();
        let second = catch_unwind(AssertUnwindSafe(rig));
        said(&*second.err().expect("the second Rig must not come"))
    });
    assert!(
        message.contains("holds the serial lock already") && message.contains("drop(rig)"),
        "{message}"
    );
}

#[test]
fn after_drop_the_next_rig_may_come() {
    guarded(Duration::from_secs(60), || {
        drop(rig());
        drop(rig());
        let _g = serial();
    });
}

#[test]
fn a_lock_held_too_long_is_red_with_a_text() {
    let (held_tx, held_rx) = mpsc::channel();
    let (free_tx, free_rx) = mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let _g = serial();
        held_tx.send(()).unwrap();
        let _ = free_rx.recv_timeout(Duration::from_secs(30));
    });
    held_rx
        .recv_timeout(Duration::from_secs(60))
        .expect("the holder got the lock");
    let message = guarded(Duration::from_secs(20), || {
        let waited = catch_unwind(AssertUnwindSafe(|| {
            serial_within(Duration::from_millis(300))
        }));
        said(&*waited.err().expect("the lock is held: no way in"))
    });
    free_tx.send(()).unwrap();
    holder.join().unwrap();
    assert!(
        message.contains("was not free for") && message.contains("another test holds it"),
        "{message}"
    );
}
