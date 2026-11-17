//! `head-bar`: the bar with buttons at its ends.
//!
//! Tests: pure value tests.
//!
//! Mutants (each must turn a test red):
//! - an element put right lands at the left -> `elements_stand_at_the_end_they_were_given_for`

use blinc_layout::div::div;
use fs_components_widgets::HeadBar;

#[test]
fn elements_stand_at_the_end_they_were_given_for() {
    let bar = HeadBar::new()
        .left(div())
        .title("Contacts")
        .right(div())
        .right(div());
    assert_eq!(bar.counts(), (1, 2));
    assert_eq!(HeadBar::new().counts(), (0, 0));
}
