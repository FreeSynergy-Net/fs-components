//! `person-face`: the circle with the initials, and the rule for a picture that cannot be loaded.
//!
//! Tests: pure value tests (what is drawn is decided before any element exists).
//!
//! Mutants (each must turn a test red):
//! - a picture that is not there is handed to Blinc (grey disc) -> `a_picture_that_cannot_be_loaded_is_never_shown`
//! - the letters are not the initials of the name              -> `the_letters_are_the_initials_of_the_name`
//! - the letters keep one size whatever the circle             -> `the_letters_grow_with_the_circle`
//! - a tiny size is taken as it is (a circle of 0 px)          -> `a_tiny_circle_is_never_smaller_than_a_dot`

use fs_components_widgets::PersonFace;

#[test]
fn the_letters_are_the_initials_of_the_name() {
    assert_eq!(PersonFace::new("Bea Quill").letters(), "BQ");
    assert_eq!(PersonFace::new("Amara").letters(), "A");
    assert_eq!(PersonFace::new("   ").letters(), "?");
}

#[test]
fn a_picture_that_cannot_be_loaded_is_never_shown() {
    let missing = PersonFace::new("Bea").image(Some("/definitely/not/here.png".into()));
    assert_eq!(missing.shown_image(), None, "the initials stand instead");
    let inline = PersonFace::new("Bea").image(Some("data:image/png;base64,AAAA".into()));
    assert_eq!(inline.shown_image(), Some("data:image/png;base64,AAAA"));
    let remote = PersonFace::new("Bea").image(Some("https://example.org/a.png".into()));
    assert!(
        remote.shown_image().is_some(),
        "an address is left to Blinc"
    );
    assert_eq!(PersonFace::new("Bea").shown_image(), None);
}

#[test]
fn the_letters_grow_with_the_circle() {
    let small = PersonFace::new("Bea").size(40.0).letter_size();
    let big = PersonFace::new("Bea").size(112.0).letter_size();
    assert!(big > 2.0 * small, "{small} {big}");
    assert!(big < 112.0 / 2.0, "the letters stay inside the circle");
}

#[test]
fn a_tiny_circle_is_never_smaller_than_a_dot() {
    assert!(PersonFace::new("Bea").size(0.0).diameter() >= 8.0);
    assert!(PersonFace::new("Bea").size(-5.0).letter_size() >= 8.0);
}
