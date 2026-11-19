//! Time zones: the names of the IANA time zone database and the check of a typed name against them.
//!
//! Pattern: Adapter over `chrono-tz` (the database as a table, compiled in; no list is written by hand here) plus Strategy for the
//! rule: the FORM of a name is asked of `fs_types::Zone::iana` (the contract crate checks the form only), whether the name EXISTS is
//! this file's, because "whether an IANA name exists is the tz database's to say" (fs-types, ADR 0085). A custom zone (`/…`, defined
//! by a `VTIMEZONE` of its object) is not a name of the database: this field does not take it. What the reader sees is the IANA name
//! itself, never an empty label (ADR 0097); the names of CLDR (`Central European Time`) are a follow-up, no CLDR package is in the stack.

use std::str::FromStr;

use chrono_tz::{TZ_VARIANTS, Tz};
use fs_types::Zone;

use crate::problem::Problem;

/// The version of the tz database inside the crate (`2025b`), for the changelog and for a program that shows where its zones come from.
pub const DATABASE_VERSION: &str = chrono_tz::IANA_TZDB_VERSION;

/// Every name of the database, in the order of the crate (`Africa/Abidjan` ...), links such as `US/Eastern` included.
pub fn names() -> Vec<&'static str> {
    TZ_VARIANTS.iter().map(|z| z.name()).collect()
}

/// The name as the database writes it (`europe/berlin` -> `Europe/Berlin`) if `typed` is one, else `None`. Names are matched exactly
/// apart from case; spaces around are cut.
pub fn canonical(typed: &str) -> Option<&'static str> {
    let t = typed.trim();
    if t.is_empty() || Zone::iana(t).is_err() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    TZ_VARIANTS
        .iter()
        .find(|z| z.name().to_ascii_lowercase() == lower)
        .map(|z| z.name())
}

/// What is wrong with a typed zone name: `None` for nothing typed or a name of the database.
pub fn check(typed: &str) -> Option<Problem> {
    if typed.trim().is_empty() {
        return None;
    }
    canonical(typed).is_none().then_some(Problem::UnknownZone)
}

/// The names that start like `typed` (any case), at most `limit`, best first (the shortest). For the list under the field.
pub fn suggest(typed: &str, limit: usize) -> Vec<&'static str> {
    let t = typed.trim().to_ascii_lowercase();
    let mut found: Vec<&'static str> = TZ_VARIANTS
        .iter()
        .map(|z| z.name())
        .filter(|n| n.to_ascii_lowercase().contains(&t))
        .collect();
    found.sort_by_key(|n| (!n.to_ascii_lowercase().starts_with(&t), n.len()));
    found.truncate(limit);
    found
}

/// The words to show for a zone: its IANA name (CLDR names are a follow-up); never empty.
pub fn label(id: &str) -> String {
    id.trim().to_owned()
}

/// The offset of a zone from UTC in seconds at the given UTC time, if the name is one (`Europe/Berlin` in July: 7200).
pub fn offset_seconds(zone: &str, utc_millis: i64) -> Option<i32> {
    use chrono::{Offset, TimeZone};
    let tz = Tz::from_str(canonical(zone)?).ok()?;
    let at = chrono::DateTime::from_timestamp_millis(utc_millis)?;
    Some(
        tz.offset_from_utc_datetime(&at.naive_utc())
            .fix()
            .local_minus_utc(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_database_is_there_and_its_names_are_valid_zones() {
        let all = names();
        assert!(all.len() > 400, "{}", all.len());
        assert!(all.contains(&"Europe/Berlin") && all.contains(&"UTC"));
        assert!(all.iter().all(|n| Zone::iana(n).is_ok()));
        assert_eq!(DATABASE_VERSION.len(), 5);
    }

    #[test]
    fn check_and_canonical() {
        assert_eq!(check(""), None);
        assert_eq!(check("  "), None);
        assert_eq!(check("Europe/Berlin"), None);
        assert_eq!(canonical(" europe/berlin "), Some("Europe/Berlin"));
        assert_eq!(check("Europe/Berlinn"), Some(Problem::UnknownZone));
        assert_eq!(check("/MyZone"), Some(Problem::UnknownZone));
        assert_eq!(check("Europe/Ber lin"), Some(Problem::UnknownZone));
    }

    #[test]
    fn suggestions_come_best_first_and_are_limited() {
        let s = suggest("berl", 5);
        assert_eq!(s.first(), Some(&"Europe/Berlin"));
        assert!(suggest("a", 3).len() == 3);
        assert!(suggest("zzzz-none", 5).is_empty());
        assert_eq!(label("Europe/Berlin"), "Europe/Berlin");
    }

    #[test]
    fn the_offset_follows_the_summer_time_of_the_database() {
        // 2026-07-01T00:00Z and 2026-01-01T00:00Z
        assert_eq!(
            offset_seconds("Europe/Berlin", 1_782_864_000_000),
            Some(7200)
        );
        assert_eq!(
            offset_seconds("Europe/Berlin", 1_767_225_600_000),
            Some(3600)
        );
        assert_eq!(offset_seconds("Nowhere/Land", 0), None);
    }
}
