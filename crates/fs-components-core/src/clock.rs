//! What "now" is for a component: the port [`Clock`] with the system clock as the default and a fixed one for tests and pictures.
//!
//! Pattern: Strategy / Port (`Clock`) with Dependency Injection through `DrawCtx` – [`SystemClock`] is the real wall clock,
//! [`FixedClock`] says one moment; a component never reads the system time itself. The unit is Unix seconds (UTC), because this
//! crate holds no date type; the host wraps its own clock in an adapter (its `Moment` to seconds).

use std::fmt::Debug;

/// Seconds in a day.
const DAY_SECONDS: i64 = 86_400;

/// Pattern: Strategy / Port – what "now" is for a component.
pub trait Clock: Send + Sync + Debug {
    /// Now, in seconds since 1970-01-01T00:00:00Z.
    fn now_unix(&self) -> i64;

    /// The day of [`Clock::now_unix`] in UTC, counted in days since 1970-01-01 (the zone is not asked here: UTC).
    fn today_days(&self) -> i64 {
        self.now_unix().div_euclid(DAY_SECONDS)
    }
}

/// The real wall clock; the default of a [`crate::form_view::DrawCtx`]. The only place of this workspace that reads the system time.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
    }
}

/// One fixed moment: always the same "now" (tests and pictures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedClock(i64);

impl FixedClock {
    /// A clock that always says `unix` seconds since 1970-01-01T00:00:00Z.
    #[must_use]
    pub fn new(unix: i64) -> Self {
        Self(unix)
    }

    /// A clock at noon UTC of the day `days` since 1970-01-01.
    #[must_use]
    pub fn at_day(days: i64) -> Self {
        Self(days * DAY_SECONDS + DAY_SECONDS / 2)
    }
}

impl Clock for FixedClock {
    fn now_unix(&self) -> i64 {
        self.0
    }
}
