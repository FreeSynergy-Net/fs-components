//! The lane of a column from the channel `lane` of its cards: all agree = that lane; none named or several = none (no guess).
//!
//! Mutant: the first lane wins -> `a_column_of_cards_from_two_lanes_has_no_lane` fails.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_kanban::columns_of;

#[test]
fn a_column_takes_the_lane_of_its_cards_and_none_when_they_name_none() {
    let input = FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![
            FormRow::of("a", &[("title", "A"), ("column", "x"), ("lane", "b1")]),
            FormRow::of("b", &[("title", "B"), ("column", "y"), ("lane", "")]),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    };
    let cols = columns_of(&input);
    assert_eq!(cols[0].lane.as_deref(), Some("b1"));
    assert_eq!(cols[1].lane, None);
}

/// Mutant: the first lane wins -> this test fails.
#[test]
fn a_column_of_cards_from_two_lanes_has_no_lane() {
    let row =
        |k: &str, c: &str, l: &str| FormRow::of(k, &[("title", k), ("column", c), ("lane", l)]);
    let input = FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![
            row("a", "x", "b1"),
            row("b", "x", "b2"),
            row("c", "x", "b1"),
            row("d", "y", "b1"),
            row("e", "y", "b1"),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    };
    let cols = columns_of(&input);
    assert_eq!(
        cols[0].lane, None,
        "mixed lanes: none, even when the first and the third agree"
    );
    assert_eq!(cols[1].lane.as_deref(), Some("b1"));
}
