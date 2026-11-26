//! Pattern: none (test). `with_week_start` sets, replaces and removes the `fw` keyword and keeps the rest of the tag.

use fs_components_words::{Locale, WEEK_START_CHOICES, with_week_start};

fn set(tag: &str, choice: &str) -> String {
    with_week_start(&Locale::parse(tag).unwrap(), choice).to_string()
}

#[test]
fn a_choice_becomes_the_fw_keyword() {
    assert_eq!(set("de", "sun"), "de-u-fw-sun");
    assert_eq!(set("en-US", "MON"), "en-US-u-fw-mon");
}

#[test]
fn auto_and_unknown_values_leave_the_week_to_the_region() {
    assert_eq!(set("de", "auto"), "de");
    assert_eq!(set("de", "someday"), "de");
    assert_eq!(
        set("de-u-fw-sun", "auto"),
        "de",
        "an earlier choice is removed"
    );
}

#[test]
fn a_new_choice_replaces_the_old_one() {
    assert_eq!(set("de-u-fw-sun", "sat"), "de-u-fw-sat");
}

#[test]
fn other_keywords_stay_and_stay_in_key_order() {
    assert_eq!(set("de-u-nu-latn", "sun"), "de-u-fw-sun-nu-latn");
    assert_eq!(
        set("de-u-ca-gregory-nu-latn", "mon"),
        "de-u-ca-gregory-fw-mon-nu-latn"
    );
    assert_eq!(set("de-u-ca-gregory-fw-sun", "auto"), "de-u-ca-gregory");
}

#[test]
fn a_following_extension_is_kept() {
    assert_eq!(set("de-u-fw-sun-x-priv", "mon"), "de-u-fw-mon-x-priv");
    assert_eq!(set("de-x-priv", "mon"), "de-u-fw-mon-x-priv");
}

#[test]
fn the_list_of_choices_starts_with_auto_and_has_the_seven_days() {
    assert_eq!(WEEK_START_CHOICES[0], "auto");
    assert_eq!(WEEK_START_CHOICES.len(), 8);
}
