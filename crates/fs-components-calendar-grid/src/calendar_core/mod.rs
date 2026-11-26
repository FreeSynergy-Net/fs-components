//! The shared core of the calendar components: the day and the moment, the reading of the form `bars-in-time`, the colour slots and what
//! every view of `bars-in-time` shares besides the reading. It moved here from the dropped `calendar-view` (ADR 0005, 0050) so that
//! `calendar-grid` and `timeline` keep one copy.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

mod bars;
mod color_slot;
mod day;
mod support;

pub use bars::{BarEntry, BarsRead, read_bars};
pub use color_slot::{SLOTS, color_slot};
pub use day::{Day, MINUTES_PER_DAY, Moment};
pub use support::{partial_notice, today};
