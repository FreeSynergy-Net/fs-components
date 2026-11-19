//! Moving along a row of things with the arrow keys: the one rule tabs, the toolbar and the accordion share.
//!
//! Pattern: Strategy — a pure function from (where, which key) to where; no element, no clock. WAI-ARIA APG (tabs, toolbar,
//! accordion): the arrow of the row's direction goes to the next, the opposite to the previous, Home to the first, End to the last;
//! at the end the move wraps when the pattern asks for it; a disabled thing is skipped.

use crate::keys::Key;

/// The way a row runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Left and Right move.
    Horizontal,
    /// Up and Down move.
    Vertical,
}

/// Where the keyboard goes from `at` in a row of `len` things when `key` is pressed, or `None` when the key is no move or nothing
/// can be reached. `skip(i)` says that thing `i` cannot be reached (disabled). `wrap` joins the end to the start.
#[allow(clippy::cast_possible_wrap)]
pub fn next(
    len: usize,
    at: Option<usize>,
    key: Key,
    axis: Axis,
    wrap: bool,
    skip: &dyn Fn(usize) -> bool,
) -> Option<usize> {
    let (back, forth) = match axis {
        Axis::Horizontal => (Key::Left, Key::Right),
        Axis::Vertical => (Key::Up, Key::Down),
    };
    let reachable = |i: usize| i < len && !skip(i);
    if key == Key::Home {
        return (0..len).find(|&i| reachable(i));
    }
    if key == Key::End {
        return (0..len).rev().find(|&i| reachable(i));
    }
    let step: isize = if key == forth {
        1
    } else if key == back {
        -1
    } else {
        return None;
    };
    let last = len as isize - 1;
    // Before the first (or after the last) when nothing has the keyboard yet: the first press goes to the first (last) thing.
    let mut i = at.map_or(if step > 0 { -1 } else { last + 1 }, |a| a as isize);
    for _ in 0..len {
        i += step;
        if i < 0 || i > last {
            if !wrap {
                return None;
            }
            i = if step > 0 { 0 } else { last };
        }
        #[allow(clippy::cast_sign_loss)]
        let found = i as usize;
        if reachable(found) {
            return Some(found);
        }
    }
    None
}
