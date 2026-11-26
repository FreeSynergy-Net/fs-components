//! `Day` and `Moment`, the value types of every calendar: counting, real dates, month steps. Pure, no window.
//!
//! Tests: Table tests over the value types.

use fs_components_calendar_grid::Day;

fn d(text: &str) -> Day {
    Day::parse(text).unwrap()
}

#[test]
fn days_count_across_centuries_and_leap_years() {
    assert_eq!(d("1970-01-01").number(), 0);
    assert_eq!(d("2000-03-01").number() - d("2000-02-28").number(), 2);
    assert_eq!(d("1900-03-01").number() - d("1900-02-28").number(), 1);
    assert_eq!(d("2026-11-06").weekday(), 4, "a Friday");
    for number in [-1_000_000, -1, 0, 59, 20_000, 2_932_896, 10_000_000] {
        assert_eq!(Day::from_number(number).number(), number);
    }
    assert_eq!(d("2026-12-31").add_days(1), d("2027-01-01"));
    assert_eq!(
        d("3000-02-28").add_days(1),
        d("3000-03-01"),
        "not a leap year"
    );
}

#[test]
fn only_real_dates_are_dates() {
    for bad in [
        "2026-02-29",
        "2026-13-01",
        "2026-00-10",
        "26-1-1",
        "2026-1-1",
        "x",
    ] {
        assert_eq!(Day::parse(bad), None, "{bad}");
    }
    assert!(Day::parse("2028-02-29").is_some());
    assert_eq!(d("2028-02-29").to_string(), "2028-02-29");
}

#[test]
fn a_month_step_from_the_31st_lands_on_the_last_of_a_short_month() {
    assert_eq!(d("2027-01-31").add_months(1), d("2027-02-28"));
    assert_eq!(d("2027-03-31").add_months(-1), d("2027-02-28"));
    assert_eq!(d("2026-12-15").add_months(1), d("2027-01-15"));
    assert_eq!(d("2027-01-15").add_months(-1), d("2026-12-15"));
}
