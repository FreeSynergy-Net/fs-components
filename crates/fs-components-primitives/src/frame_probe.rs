//! Counters for the frame log (`FS_FRAME_LOG`, see `fs-components-app`): what the program's own build code
//! reports about one frame. Probe code, switched on by the frame log only.
//!
//! Pattern: Observer (the frame log reads, the build code writes) over plain atomics.
//!
//! Until [`switch_on`] is called every function here returns at once (one relaxed load): no clock is read,
//! nothing is counted. The frame log calls [`take`] once per frame row, which empties the counters, so a value
//! always belongs to the frame since the last row.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

static ON: AtomicBool = AtomicBool::new(false);
static BUILD_US: AtomicU64 = AtomicU64::new(0);
static CARDS_US: AtomicU64 = AtomicU64::new(0);
static CARDS_N: AtomicU64 = AtomicU64::new(0);
static STATEFUL_N: AtomicU64 = AtomicU64::new(0);

/// What was reported since the last [`take`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// Microseconds our build closure ran (summed over the builds since the last row).
    pub build_us: u64,
    /// Microseconds spent creating the cards of a list (only when they were really computed, not reused).
    pub cards_us: u64,
    /// Number of cards of the last list created.
    pub cards_n: u64,
    /// Stateful widgets (dropdown, tooltip) constructed.
    pub stateful_n: u64,
}

/// Start counting. Called once by the frame log when it is installed.
pub fn switch_on() {
    ON.store(true, Ordering::Relaxed);
}

/// Is counting on.
pub fn is_on() -> bool {
    ON.load(Ordering::Relaxed)
}

fn micros(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

/// Run `f` (our build closure) and add its duration to `build_us`.
pub fn time_build<R>(f: impl FnOnce() -> R) -> R {
    if !is_on() {
        return f();
    }
    let start = Instant::now();
    let result = f();
    BUILD_US.fetch_add(micros(start), Ordering::Relaxed);
    result
}

/// Run `f` (creates the cards of a list) and add its duration to `cards_us`; remember the length as `cards_n`.
pub fn time_cards<T>(f: impl FnOnce() -> Vec<T>) -> Vec<T> {
    if !is_on() {
        return f();
    }
    let start = Instant::now();
    let result = f();
    CARDS_US.fetch_add(micros(start), Ordering::Relaxed);
    CARDS_N.store(result.len() as u64, Ordering::Relaxed);
    result
}

/// One stateful widget (dropdown, tooltip) is being constructed.
pub fn count_stateful() {
    if is_on() {
        STATEFUL_N.fetch_add(1, Ordering::Relaxed);
    }
}

/// Read and empty the counters.
pub fn take() -> Snapshot {
    Snapshot {
        build_us: BUILD_US.swap(0, Ordering::Relaxed),
        cards_us: CARDS_US.swap(0, Ordering::Relaxed),
        cards_n: CARDS_N.swap(0, Ordering::Relaxed),
        stateful_n: STATEFUL_N.swap(0, Ordering::Relaxed),
    }
}
