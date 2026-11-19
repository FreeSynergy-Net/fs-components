//! `FieldSlot`: the block name decides the component; a block without a component shows the value itself, never nothing.
//! Mutants: `Kind::block` and `from_block` disagree -> `blocks_round_trip`; `line` of an unknown block empty -> `no_component_yet`.
//! Pattern: none; tests of pure functions.

#![allow(clippy::pedantic)]

use fs_components_fields::{FieldSlot, Kind};

#[test]
fn blocks_round_trip() {
    for kind in Kind::ALL {
        assert_eq!(Kind::from_block(kind.block()), Some(kind));
        // the block name is the key in kebab case (`dateTime` -> `field-date-time`)
        let kebab: String = kind
            .field()
            .key()
            .chars()
            .flat_map(|c| {
                if c.is_ascii_uppercase() {
                    vec!['-', c.to_ascii_lowercase()]
                } else {
                    vec![c]
                }
            })
            .collect();
        assert_eq!(kind.block(), format!("field-{kebab}"));
    }
    assert_eq!(Kind::from_block("field-text"), None);
}

#[test]
fn a_slot_says_its_value_in_the_words_of_its_type() {
    let s = FieldSlot::new("field-email", "label.email", "ada@example.org");
    assert_eq!(s.kind(), Some(Kind::Email));
    assert_eq!(s.line(), Kind::Email.field().compact(&s.parts()));
}

#[test]
fn no_component_yet() {
    let s = FieldSlot::new("field-integer", "label.count", "42");
    assert_eq!(s.kind(), None);
    assert_eq!(s.parts(), vec!["42".to_owned()]);
    assert_eq!(s.line(), "42");
}
