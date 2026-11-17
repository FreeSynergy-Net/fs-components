//! The beat of live changes (b7 decision O7): the shell asks [`Settings::refresh`] on a timer, the interval is a setting
//! (`fs.shell.refresh-ms`, [`crate::REFRESH`]), and it asks ONLY while a window is open (a program with no window
//! reads no file).
//!
//! No clock here: the shell passes `now_ms`, any monotonic millisecond count.
//!
//! Pattern: State object (counts windows, remembers the last beat) + pure decision ([`Pace::due`]).

use crate::resolve::Effective;
use crate::settings::{Change, Settings};
use crate::standard::{REFRESH, ShellSettings};
use crate::value::Value;

/// When to ask for changes.
#[derive(Clone, Debug)]
pub struct Pace {
    interval_ms: u64,
    windows: usize,
    last: Option<u64>,
}

impl Pace {
    /// The smallest interval (ms).
    pub const MIN_MS: u64 = 250;
    /// The largest interval (ms).
    pub const MAX_MS: u64 = 60_000;
    /// The default interval (ms).
    pub const DEFAULT_MS: u64 = 1000;

    /// A pace of `interval_ms` (kept between [`Pace::MIN_MS`] and [`Pace::MAX_MS`]), no window open.
    #[must_use]
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval_ms: interval_ms.clamp(Self::MIN_MS, Self::MAX_MS),
            windows: 0,
            last: None,
        }
    }

    /// The interval now (ms).
    #[must_use]
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    /// How many windows are open.
    #[must_use]
    pub fn windows(&self) -> usize {
        self.windows
    }

    /// A window of the program opened.
    pub fn window_opened(&mut self) {
        self.windows += 1;
    }

    /// A window of the program closed; with the last one the beat stops and starts anew with the next window.
    pub fn window_closed(&mut self) {
        self.windows = self.windows.saturating_sub(1);
        if self.windows == 0 {
            self.last = None;
        }
    }

    /// Whether to ask at `now_ms`: never without a window; else at the first call and then every interval.
    pub fn due(&mut self, now_ms: u64) -> bool {
        if self.windows == 0 {
            return false;
        }
        let due = self
            .last
            .is_none_or(|last| now_ms.saturating_sub(last) >= self.interval_ms);
        if due {
            self.last = Some(now_ms);
        }
        due
    }

    /// The beat: takes the interval from the setting (so a change of it is live too), and refreshes `settings` when due.
    pub fn tick(&mut self, now_ms: u64, settings: &Settings) -> Vec<Change> {
        if let Some(Effective {
            value: Value::Whole(ms),
            ..
        }) = ShellSettings::key(REFRESH)
            .ok()
            .and_then(|k| settings.get(&k))
        {
            self.interval_ms = u64::try_from(ms)
                .unwrap_or(Self::DEFAULT_MS)
                .clamp(Self::MIN_MS, Self::MAX_MS);
        }
        if self.due(now_ms) {
            settings.refresh()
        } else {
            Vec::new()
        }
    }
}

impl Default for Pace {
    fn default() -> Self {
        Self::new(Self::DEFAULT_MS)
    }
}
