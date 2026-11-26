//! `FormRow` keeps the path of every value: a reader asks for the values of a FIELD, never for the value at a position.
//!
//! Pattern: none; tests (fixed cases of the Adapter `FormRow::from_values` and a property over it).
#![allow(clippy::pedantic)]

use fs_components_core::form_read::{FieldInfo, FormRow};
use proptest::prelude::*;

fn map() -> Vec<(String, Vec<String>)> {
    vec![(
        "columns".to_owned(),
        ["organizations", "emails", "phones"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
    )]
}

#[test]
fn two_emails_stay_under_emails_and_a_missing_field_shifts_nothing() {
    // the module leaves `organizations` out (RFC 9553: no entry, no value)
    let wire = [
        ("emails", "a@x.org"),
        ("emails", "b@x.org"),
        ("phones", "+49 1"),
    ];
    let row = FormRow::from_values("k", Vec::new(), &map(), wire);
    assert_eq!(row.field("columns", "organizations"), Vec::<&str>::new());
    assert_eq!(row.field("columns", "emails"), ["a@x.org", "b@x.org"]);
    assert_eq!(row.field("columns", "phones"), ["+49 1"]);
    // `all` still lists the texts of the channel in the order of the map
    assert_eq!(row.all("columns"), ["a@x.org", "b@x.org", "+49 1"]);
}

#[test]
fn an_empty_text_is_a_value_of_its_field_and_gets_no_neighbour_s_place() {
    let row = FormRow::from_values(
        "k",
        Vec::new(),
        &map(),
        [("organizations", ""), ("phones", "1")],
    );
    assert_eq!(row.field("columns", "organizations"), [""]);
    assert_eq!(row.field("columns", "phones"), ["1"]);
}

#[test]
fn a_value_of_a_field_the_map_does_not_name_is_never_seen() {
    let row = FormRow::from_values(
        "k",
        Vec::new(),
        &map(),
        [("nicknames", "Mara"), ("emails", "e")],
    );
    assert_eq!(row.all("columns"), ["e"]);
    assert!(row.field("columns", "nicknames").is_empty());
}

#[test]
fn a_field_in_two_channels_fills_both() {
    let map = vec![
        ("facts".to_owned(), vec!["who".to_owned()]),
        ("assignee".to_owned(), vec!["who".to_owned()]),
    ];
    let row = FormRow::from_values("k", vec!["card.move".to_owned()], &map, [("who", "Ada")]);
    assert_eq!(row.field("facts", "who"), ["Ada"]);
    assert_eq!(row.field("assignee", "who"), ["Ada"]);
    assert_eq!(row.may, ["card.move"]);
}

#[test]
fn the_words_of_a_field_are_its_sentence_else_its_name() {
    let f = FieldInfo {
        name: "emails".into(),
        block: "field-email".into(),
        label: "person.emails".into(),
    };
    let say = |k: &str| {
        if k == "person.emails" {
            "E-mail".to_owned()
        } else {
            k.to_owned()
        }
    };
    assert_eq!(f.shown(&say), "E-mail");
    assert_eq!(
        f.shown(&|k: &str| k.to_owned()),
        "emails",
        "no sentence: the name"
    );
    let none = FieldInfo {
        label: String::new(),
        ..f
    };
    assert_eq!(
        none.shown(&say),
        "emails",
        "no word id: the name, never `channel.emails`"
    );
}

proptest! {
    /// Whatever the module sends (any interleaving, 0..3 values per field), `field` returns exactly the values of that field in the
    /// order they came, and the other fields' values never mix in.
    #[test]
    fn the_values_of_a_field_are_exactly_the_values_with_its_path(wire in prop::collection::vec(0usize..3, 0..10)) {
        let names = ["organizations", "emails", "phones"];
        let mut n = [0usize; 3];
        let pairs: Vec<(String, String)> = wire.iter().map(|&i| {
            let t = format!("{i}-{}", n[i]); n[i] += 1; (names[i].to_owned(), t)
        }).collect();
        let row = FormRow::from_values("k", Vec::new(), &map(), pairs.iter().map(|(p, t)| (p.as_str(), t.as_str())));
        for (i, name) in names.iter().enumerate() {
            let want: Vec<String> = (0..n[i]).map(|j| format!("{i}-{j}")).collect();
            prop_assert_eq!(row.field("columns", name), want.iter().map(String::as_str).collect::<Vec<_>>());
        }
    }
}
