//! Pattern: none (test). Without a package the names are neutral (Null Object): ISO forms, week on Monday, no language table;
//! also for a package whose bytes are no blob.

use std::sync::Arc;

use fs_components_mf2::{DateData, DateDir, NeutralDateNames, NoDates, date_names};
use fs_components_words::{DateNames, Locale, Weekday, Width};

fn loc(tag: &str) -> Locale {
    Locale::parse(tag).unwrap()
}

#[test]
fn no_package_gives_the_neutral_names() {
    let n = date_names(&NoDates, &loc("de"));
    assert_eq!(n.weekday(Weekday::Monday, Width::Long), "Mon");
    assert_eq!(n.weekday(Weekday::Sunday, Width::Short), "Sun");
    assert_eq!(n.month(11, Width::Long), "11");
    assert_eq!(n.month_title(2026, 11, Width::Long), "2026-11");
    assert_eq!(n.day_title(2026, 11, 26, Width::Long), "Thu 2026-11-26");
    assert_eq!(n.first_weekday(), Weekday::Monday);
}

#[test]
fn a_language_without_a_package_is_neutral_next_to_ones_with() {
    let dir =
        DateDir::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/packed"));
    assert_eq!(date_names(&dir, &loc("ja")).month(11, Width::Long), "11");
    assert_eq!(
        date_names(&dir, &loc("de")).month(11, Width::Long),
        "November"
    );
}

#[test]
fn a_broken_blob_is_neutral() {
    struct Broken;
    impl DateData for Broken {
        fn blob(&self, _tag: &Locale) -> Option<Arc<[u8]>> {
            None
        }
        fn names(&self, _tag: &Locale) -> Option<Arc<[u8]>> {
            Some(Arc::from(&b"no blob"[..]))
        }
    }
    assert_eq!(date_names(&Broken, &loc("de")).month(11, Width::Long), "11");
}

#[test]
fn the_neutral_names_alone_need_no_data() {
    let n = NeutralDateNames;
    assert_eq!(n.weekday(Weekday::Thursday, Width::Medium), "Thu");
    assert_eq!(n.day_title(2024, 2, 29, Width::Long), "Thu 2024-02-29");
    assert_eq!(
        n.day_title(2026, 2, 29, Width::Long),
        "2026-02-29",
        "no such day, no weekday"
    );
}
