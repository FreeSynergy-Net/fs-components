//! The sample value of every kind, as parts, for the tests that draw every component.
#![allow(clippy::pedantic)]

use fs_components_fields::Kind;

pub fn sample(kind: Kind) -> Vec<String> {
    let v: &[&str] = match kind {
        Kind::Email => &["ada", "example.org"],
        Kind::Name => &["Dr.", "Ada", "Augusta", "Lovelace", ""],
        Kind::Phone => &["+49", "30 1234567"],
        Kind::Address => &["Main St 1", "10115", "Berlin", "", "Germany"],
        Kind::Date => &["2026", "11", "10"],
        Kind::Choice => &["blue"],
        Kind::Keywords => &["family, work"],
        Kind::Reference => &["https://example.org/tasks/7"],
        Kind::Link => &["https://example.org/page"],
        Kind::Multiline => &["First line", "", ""][..1],
        Kind::Flag => &["true"],
        Kind::Integer => &["-42"],
        Kind::Whole => &["42"],
    };
    v.iter().map(|s| (*s).to_string()).collect()
}
