//! `form_read::percent`: the one reading of a percentage channel (kanban cards, calendar and timeline bars).
//!
//! Tests: Table test (rounding, cutting, text, not finite).
use fs_components_core::form_read::percent;

#[test]
fn a_number_is_rounded_and_cut_to_0_to_100() {
    assert_eq!(percent("33.3"), Some(33));
    assert_eq!(percent("99.6"), Some(100));
    assert_eq!(percent("101"), Some(100));
    assert_eq!(percent("-5"), Some(0));
    assert_eq!(percent(" 7 "), Some(7));
}

#[test]
fn text_and_non_finite_are_no_number() {
    assert_eq!(percent("abc"), None);
    assert_eq!(percent(""), None);
    assert_eq!(percent("inf"), None);
    assert_eq!(percent("NaN"), None);
}
