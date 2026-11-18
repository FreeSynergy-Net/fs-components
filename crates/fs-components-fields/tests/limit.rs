//! `Limit`: a count of at least one, read from a setting; the declaration every block uses (rule 57).
//!
//! Mutants: `Limit::new(0)` is a limit -> `zero_is_no_limit`; `read` ignores the setting -> `read_takes_the_effective_value`; the
//! kind allows 0 -> `the_declaration_is_a_whole_number_from_one`; the default is written wrong -> same test.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_fields::Limit;
use fs_components_fields::limit::declaration;
use fs_settings::{
    Catalog, GroupId, Kind, MemoryStore, NoSystem, Owner, OwnerKind, ProgramName, SettingKey,
    Settings, Target, Value,
};

fn key(local: &str) -> SettingKey {
    SettingKey::new(&Owner::new(OwnerKind::Framework, "probe").unwrap(), local).unwrap()
}

fn settings_with(local: &str, default: usize) -> Settings {
    let d = declaration(
        key(local),
        Limit::of(default),
        GroupId::new("probe").unwrap(),
    );
    Settings::open(
        ProgramName::new("probe").unwrap(),
        Catalog::of(vec![d]).unwrap(),
        Arc::new(MemoryStore::new()),
        Arc::new(NoSystem),
    )
}

#[test]
fn zero_is_no_limit() {
    assert_eq!(Limit::new(0), None);
    assert_eq!(Limit::new(1).map(Limit::get), Some(1));
    assert_eq!(Limit::new(usize::MAX).map(Limit::get), Some(usize::MAX));
}

#[test]
#[should_panic(expected = "at least one")]
fn a_limit_of_zero_panics() {
    let zero = std::hint::black_box(0);
    let _ = Limit::of(zero);
}

#[test]
fn the_declaration_is_a_whole_number_from_one() {
    let d = declaration(key("a"), Limit::of(9), GroupId::new("probe").unwrap());
    assert_eq!(d.default, Value::Whole(9));
    assert_eq!(
        d.kind,
        Kind::Whole {
            min: 1,
            max: i64::from(i32::MAX)
        }
    );
    assert!(d.kind.fits(&key("a"), &Value::Whole(1)));
    assert!(d.kind.fits(&key("a"), &Value::Whole(i64::from(i32::MAX))));
    assert!(!d.kind.fits(&key("a"), &Value::Whole(0)));
    assert!(
        !d.kind
            .fits(&key("a"), &Value::Whole(i64::from(i32::MAX) + 1))
    );
}

#[test]
fn read_takes_the_effective_value() {
    let s = settings_with("a", 9);
    let k = key("a");
    assert_eq!(
        Limit::read(&s, &k, Limit::of(3)).get(),
        9,
        "the declared default, not the caller's fallback"
    );
    s.save(&k, Some(&Value::Whole(2)), Target::ThisProgram)
        .unwrap();
    assert_eq!(Limit::read(&s, &k, Limit::of(3)).get(), 2);
    assert_eq!(
        Limit::read(&s, &key("other"), Limit::of(3)).get(),
        3,
        "not declared: the fallback"
    );
}
