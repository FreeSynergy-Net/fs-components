//! Tests of the `DateNames` interface types only (no implementation lives in this crate).
//!
//! Pattern: Strategy (Provider) – a test double stands in for the provider and proves that the
//! trait can be held as a trait object.

use std::sync::Arc;

use fs_components_words::{DateNames, Weekday, Width};

struct Fixed;

impl DateNames for Fixed {
    fn weekday(&self, day: Weekday, _: Width) -> String {
        format!("d{}", day as u8)
    }
    fn month(&self, month: u8, _: Width) -> String {
        format!("m{month}")
    }
    fn month_title(&self, year: i32, month: u8, _: Width) -> String {
        format!("{year}-{month}")
    }
    fn day_title(&self, year: i32, month: u8, day: u8, _: Width) -> String {
        format!("{year}-{month}-{day}")
    }
    fn first_weekday(&self) -> Weekday {
        Weekday::Monday
    }
}

#[test]
fn weekdays_are_numbered_as_in_iso_8601() {
    let all = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];
    let numbers: Vec<u8> = all.iter().map(|d| *d as u8).collect();
    assert_eq!(numbers, vec![1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn widths_run_from_long_to_short() {
    assert!(Width::Long < Width::Medium && Width::Medium < Width::Short);
}

#[test]
fn the_trait_is_object_safe_and_shareable() {
    let names: Arc<dyn DateNames> = Arc::new(Fixed);
    let other = Arc::clone(&names);
    let joined = std::thread::spawn(move || other.weekday(Weekday::Sunday, Width::Long));
    assert_eq!(joined.join().unwrap(), "d7");
    assert_eq!(names.first_weekday(), Weekday::Monday);
    assert_eq!(names.month_title(2026, 11, Width::Long), "2026-11");
    assert_eq!(names.day_title(2026, 11, 23, Width::Short), "2026-11-23");
    assert_eq!(names.month(3, Width::Medium), "m3");
}
