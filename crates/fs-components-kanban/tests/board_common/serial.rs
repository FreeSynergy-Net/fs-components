//! The lock of the board tests that share Blinc's global state.
//!
//! Pattern: Mutex guard (RAII) – one test at a time holds the lock; a second take by the same thread panics instead of hanging.
#![allow(dead_code, clippy::pedantic)]

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// How long a test waits for the other tests that hold the lock (they draw for milliseconds; a minute means a hang).
pub const SERIAL_WAIT: Duration = Duration::from_secs(120);

thread_local! {
    /// This thread (= this test) holds the lock already.
    static HOLDS: Cell<bool> = const { Cell::new(false) };
}

/// The lock of the tests that share Blinc's global state; released (and the thread's mark cleared) when dropped.
pub struct Serial {
    _guard: MutexGuard<'static, ()>,
}

impl Drop for Serial {
    fn drop(&mut self) {
        HOLDS.with(|h| h.set(false));
    }
}

/// Takes the lock within `wait`. Panics with a plain text, never hangs: (1) the same test holds it already (a second `Rig` or
/// `serial()` while the first is alive would wait for itself for ever: `drop` the first), (2) another test holds it for longer
/// than `wait`.
pub fn serial_within(wait: Duration) -> Serial {
    assert!(
        !HOLDS.with(Cell::get),
        "this test holds the serial lock already (a second Rig or serial() while the first is alive would wait for itself): drop(rig) first"
    );
    let start = Instant::now();
    loop {
        match ONE_AT_A_TIME.try_lock() {
            Ok(guard) => return mark(guard),
            Err(TryLockError::Poisoned(p)) => return mark(p.into_inner()),
            Err(TryLockError::WouldBlock) => {
                assert!(
                    start.elapsed() < wait,
                    "the serial lock of the board tests was not free for {wait:?}: another test holds it (a hang?)"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

fn mark(guard: MutexGuard<'static, ()>) -> Serial {
    HOLDS.with(|h| h.set(true));
    Serial { _guard: guard }
}

pub fn serial() -> Serial {
    serial_within(SERIAL_WAIT)
}
