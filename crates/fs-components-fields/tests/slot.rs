//! `FieldSlot`: the block name decides the component; a block without a component shows the value itself, never nothing.
//! Mutants: `Kind::block` and `from_block` disagree -> `blocks_round_trip`; `line` of an unknown block empty -> `no_component_yet`.
//! Pattern: none; tests of pure functions.

#![allow(clippy::pedantic)]

use fs_components_fields::{FieldSlot, Kind};

#[test]
fn blocks_round_trip() {
    for kind in Kind::ALL {
        assert_eq!(Kind::from_block(kind.block()), Some(kind));
        assert_eq!(kind.block(), format!("field-{}", kind.field().key()));
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
