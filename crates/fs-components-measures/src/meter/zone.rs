//! The zone of a meter value.
//!
//! Pattern: Strategy — a pure function; the rule of HTML's `meter` element: the range is cut at `low` and `high` into three regions,
//! the region of `optimum` is the good one, the next to it is fair, the far one is poor.

/// How good a value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    /// In the region of the optimum.
    Good,
    /// In a region next to it.
    Fair,
    /// In the region farthest from it.
    Poor,
}

/// Which of the three regions a value is in: 0 below `low`, 1 between, 2 above `high`.
fn region(value: f64, low: f64, high: f64) -> u8 {
    if value < low {
        0
    } else if value > high {
        2
    } else {
        1
    }
}

/// The zone of `value` when the regions are cut at `low` and `high` and the best place is `optimum`.
pub fn zone(value: f64, low: f64, high: f64, optimum: f64) -> Zone {
    let (low, high) = if low <= high {
        (low, high)
    } else {
        (high, low)
    };
    let here = i16::from(region(value, low, high));
    let best = i16::from(region(optimum, low, high));
    match (here - best).abs() {
        0 => Zone::Good,
        1 => Zone::Fair,
        _ => Zone::Poor,
    }
}
