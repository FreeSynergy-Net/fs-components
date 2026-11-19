//! `roving`: the pure rule for moving along a row with the arrow keys (tabs, toolbar).
//!
//! Tests: model tests over `next`; no element, no window. Each branch (direction, ends, wrap, skip, nothing on the row) has its own line.

use fs_components_widgets::keys::Key;
use fs_components_widgets::roving::{Axis, next};

const H: Axis = Axis::Horizontal;
const V: Axis = Axis::Vertical;

fn none(_: usize) -> bool {
    false
}

#[test]
fn the_arrow_of_the_row_goes_forth_and_back_and_the_other_axis_does_nothing() {
    assert_eq!(next(4, Some(1), Key::Right, H, false, &none), Some(2));
    assert_eq!(next(4, Some(1), Key::Left, H, false, &none), Some(0));
    assert_eq!(next(4, Some(1), Key::Down, H, false, &none), None);
    assert_eq!(next(4, Some(1), Key::Up, H, false, &none), None);
    assert_eq!(next(4, Some(1), Key::Down, V, false, &none), Some(2));
    assert_eq!(next(4, Some(1), Key::Up, V, false, &none), Some(0));
    assert_eq!(next(4, Some(1), Key::Right, V, false, &none), None);
    assert_eq!(next(4, Some(1), Key::Left, V, false, &none), None);
}

#[test]
fn home_and_end_jump_to_the_first_and_last_that_can_be_reached() {
    let skip = |i: usize| i == 0 || i == 3;
    assert_eq!(next(4, Some(2), Key::Home, H, false, &none), Some(0));
    assert_eq!(next(4, Some(1), Key::End, H, false, &none), Some(3));
    assert_eq!(next(4, Some(2), Key::Home, H, false, &skip), Some(1));
    assert_eq!(next(4, Some(1), Key::End, H, false, &skip), Some(2));
}

#[test]
fn the_end_stops_without_wrap_and_joins_with_wrap() {
    assert_eq!(next(3, Some(2), Key::Right, H, false, &none), None);
    assert_eq!(next(3, Some(0), Key::Left, H, false, &none), None);
    assert_eq!(next(3, Some(2), Key::Right, H, true, &none), Some(0));
    assert_eq!(next(3, Some(0), Key::Left, H, true, &none), Some(2));
}

#[test]
fn a_thing_that_cannot_be_reached_is_skipped_also_over_the_join() {
    let skip = |i: usize| i == 1 || i == 2;
    assert_eq!(next(4, Some(0), Key::Right, H, false, &skip), Some(3));
    assert_eq!(next(4, Some(3), Key::Left, H, false, &skip), Some(0));
    assert_eq!(next(4, Some(3), Key::Right, H, true, &skip), Some(0));
    let end_skipped = |i: usize| i == 3;
    assert_eq!(next(4, Some(2), Key::Right, H, true, &end_skipped), Some(0));
    assert_eq!(next(4, Some(2), Key::Right, H, false, &end_skipped), None);
}

#[test]
fn nothing_to_reach_gives_nothing_and_a_key_that_is_no_move_is_ignored() {
    let all = |_: usize| true;
    assert_eq!(next(3, Some(0), Key::Right, H, true, &all), None);
    assert_eq!(next(3, None, Key::Home, H, true, &all), None);
    assert_eq!(next(0, None, Key::Right, H, true, &none), None);
    assert_eq!(next(3, Some(0), Key::Enter, H, true, &none), None);
    assert_eq!(next(3, Some(0), Key::Other(65), H, true, &none), None);
}

#[test]
fn with_no_cursor_yet_a_forth_key_goes_to_the_first_and_a_back_key_to_the_last() {
    assert_eq!(next(3, None, Key::Right, H, false, &none), Some(0));
    assert_eq!(next(3, None, Key::Left, H, false, &none), Some(2));
}
