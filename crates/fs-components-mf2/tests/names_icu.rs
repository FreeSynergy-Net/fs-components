//! Pattern: none (test). `IcuDateNames` reads the `names.blob` of the packed test data (`tests/data/packed/dates/{de,en,fr}`):
//! names, titles, the week start by region and by `-u-fw`, the trait object, and the fallbacks. Each rule has its counter-probe.

use std::sync::Arc;

use fs_components_mf2::{DateDir, date_names};
use fs_components_words::{DateNames, Locale, Weekday, Width};

fn names(tag: &str) -> Arc<dyn DateNames> {
    let dir =
        DateDir::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/packed"));
    date_names(&dir, &Locale::parse(tag).unwrap())
}

#[test]
fn german_names_and_titles() {
    let n = names("de");
    assert_eq!(n.month(11, Width::Long), "November");
    assert_eq!(n.month_title(2026, 11, Width::Long), "November 2026");
    assert_eq!(n.weekday(Weekday::Monday, Width::Long), "Montag");
    assert_eq!(n.weekday(Weekday::Sunday, Width::Long), "Sonntag");
    assert_eq!(n.weekday(Weekday::Monday, Width::Short), "Mo");
    assert_eq!(n.day_title(2026, 11, 26, Width::Medium), "Do., 26. Nov.");
    assert_eq!(
        n.day_title(2026, 11, 26, Width::Long),
        "Donnerstag, 26. November"
    );
    assert_eq!(n.first_weekday(), Weekday::Monday);
}

#[test]
fn english_names_and_titles() {
    let n = names("en");
    assert_eq!(n.month(11, Width::Long), "November");
    assert_eq!(n.month_title(2026, 11, Width::Long), "November 2026");
    assert_eq!(n.weekday(Weekday::Thursday, Width::Long), "Thursday");
    assert_eq!(n.weekday(Weekday::Thursday, Width::Medium), "Thu");
    assert_eq!(n.day_title(2026, 11, 26, Width::Medium), "Thu, Nov 26");
}

#[test]
fn french_names_and_titles() {
    let n = names("fr");
    assert_eq!(n.month(11, Width::Long), "novembre");
    assert_eq!(n.month_title(2026, 11, Width::Long), "novembre 2026");
    assert_eq!(n.weekday(Weekday::Monday, Width::Long), "lundi");
    assert!(
        n.day_title(2026, 11, 26, Width::Long)
            .starts_with("jeudi 26")
    );
    assert_eq!(n.first_weekday(), Weekday::Monday);
}

#[test]
fn the_week_starts_by_region_and_by_choice() {
    assert_eq!(
        names("en-US").first_weekday(),
        Weekday::Sunday,
        "United States: Sunday"
    );
    assert_eq!(
        names("en-GB").first_weekday(),
        Weekday::Monday,
        "the full locale is read first"
    );
    assert_eq!(names("de-AT").first_weekday(), Weekday::Monday);
    // The person's choice is a BCP 47 extension and wins over the region.
    assert_eq!(names("de-u-fw-sun").first_weekday(), Weekday::Sunday);
    assert_eq!(names("en-GB-u-fw-sat").first_weekday(), Weekday::Saturday);
}

#[test]
fn a_region_without_data_reads_the_language_alone() {
    // `de-DE` is no key in the blob: the rule falls back to `de`, with the same names.
    let n = names("de-DE");
    assert_eq!(
        n.first_weekday(),
        Weekday::Monday,
        "the week data has no de-DE key either"
    );
    assert_eq!(n.month_title(2026, 11, Width::Long), "November 2026");
    assert_eq!(
        names("en-US").weekday(Weekday::Monday, Width::Long),
        "Monday"
    );
    // A region CLDR does not know: the language alone answers (English: the Sunday of the likely region), not the neutral names.
    let xx = names("en-XX");
    assert_eq!(xx.weekday(Weekday::Monday, Width::Long), "Monday");
    assert_eq!(xx.first_weekday(), Weekday::Sunday);
}

#[test]
fn a_value_that_does_not_exist_is_neutral_not_a_panic() {
    let n = names("de");
    assert_eq!(n.month(13, Width::Long), "13");
    assert_eq!(n.month_title(2026, 13, Width::Long), "2026-13");
    assert_eq!(n.day_title(2026, 2, 30, Width::Long), "2026-02-30");
}

#[test]
fn the_trait_object_is_shared_between_threads() {
    let n = names("de");
    let other = Arc::clone(&n);
    let said = std::thread::spawn(move || other.month(11, Width::Long))
        .join()
        .unwrap();
    assert_eq!(said, n.month(11, Width::Long));
}
