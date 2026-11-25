//! Tests of the reading of a form input into cards and columns.
//!
//! Pattern: none (tests).

use fs_components_core::form_read::{ChannelInfo, FieldInfo};
use fs_components_core::states::Load;

use super::*;

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "cards-in-columns@1.0.0".into(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
    }
}

fn card(key: &str, column: &str, order: Option<&str>) -> FormRow {
    let mut pairs = vec![("title", key), ("column", column)];
    if let Some(o) = order {
        pairs.push(("order", o));
    }
    FormRow::of(key, &pairs)
}

fn keys(columns: &[Column], at: usize) -> Vec<String> {
    columns[at].cards.iter().map(|c| c.key.clone()).collect()
}

#[test]
fn the_board_is_made_from_the_channels_column_and_title_in_order_of_appearance() {
    let cols = columns_of(&input(vec![
        FormRow::of("a", &[("title", "One"), ("column", "To do")]),
        FormRow::of("b", &[("title", "Two"), ("column", "Doing")]),
        FormRow::of("c", &[("title", "Three"), ("column", "To do")]),
        FormRow::of("d", &[("title", "No column")]),
    ]));
    let names: Vec<_> = cols.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(names, ["To do", "Doing"]);
    assert_eq!(keys(&cols, 0), ["a", "c"]);
    assert_eq!(keys(&cols, 1), ["b"]);
    let none = columns_of(&input(Vec::new()));
    assert_eq!(none.len(), usize::from(false));
}

#[test]
fn the_key_of_a_column_is_its_value_and_its_name_is_the_label() {
    let cols = columns_of(&input(vec![FormRow::of(
        "a",
        &[
            ("title", "A"),
            ("column", "To-Do_1"),
            ("column-label", "To do"),
        ],
    )]));
    assert_eq!(cols[0].key, "To-Do_1", "no slug: a move names this key");
    assert_eq!(cols[0].title, "To do");
}

#[test]
fn a_column_the_module_names_is_drawn_without_a_card_and_in_its_place() {
    let mut i = input(vec![card("a", "doing", None)]);
    i.channels.push(ChannelInfo {
        name: "column".into(),
        fields: Vec::new(),
        choices: vec![
            ("todo".into(), "To do".into()),
            ("doing".into(), "In progress".into()),
            ("closed".into(), "Closed".into()),
        ],
    });
    let cols = columns_of(&i);
    let ks: Vec<_> = cols.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(ks, ["todo", "doing", "closed"]);
    assert!(cols[2].cards.is_empty() && cols[2].title == "Closed");
    assert_eq!(cols[1].cards.len(), 1);
}

#[test]
fn a_card_has_its_bar_and_its_facts_in_the_order_of_the_map() {
    let mut a = card("a", "todo", None);
    a.values.push(("progress".into(), vec!["40".into()]));
    a.values.push((
        "facts".into(),
        vec!["2026-11-26T12:00:00".into(), "high".into()],
    ));
    let mut b = card("b", "doing", None);
    b.values.push(("progress".into(), vec!["250".into()]));
    b.values
        .push(("facts".into(), vec![String::new(), "low".into()]));
    let mut i = input(vec![a, b]);
    let field = |n: &str, b: &str| FieldInfo {
        name: n.into(),
        block: b.into(),
    };
    i.channels.push(ChannelInfo {
        name: "facts".into(),
        fields: vec![field("due", "field-date-time"), field("prio", "field-text")],
        choices: Vec::new(),
    });
    let cols = columns_of(&i);
    let a = &cols[0].cards[0];
    assert_eq!(a.progress, Some(40));
    let facts: Vec<_> = a
        .facts()
        .iter()
        .map(|f| (f.block.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        [
            ("field-date-time", "2026-11-26T12:00:00"),
            ("field-text", "high")
        ]
    );
    let b = &cols[1].cards[0];
    assert_eq!(b.progress, Some(100), "more than 100 is cut to 100");
    assert_eq!(b.facts().len(), 2, "a fact without a value keeps its place");
    assert_eq!(b.shown_facts().count(), 1, "but it is not drawn");
}

#[test]
fn the_channel_column_limit_sets_the_wip_limit_of_its_column_and_a_bad_value_sets_none() {
    let with = |key: &str, col: &str, lim: &str| {
        FormRow::of(
            key,
            &[("title", key), ("column", col), ("column-limit", lim)],
        )
    };
    let cols = columns_of(&input(vec![
        card("a", "todo", None),
        with("b", "todo", "3"),
        with("c", "doing", "many"),
        with("d", "closed", "-1"),
    ]));
    assert_eq!(
        cols[0].limit,
        Some(3),
        "a row without it does not hide the one that has it"
    );
    assert_eq!(cols[1].limit, None);
    assert_eq!(cols[2].limit, None);
}

#[test]
fn a_module_that_delivers_unsorted_is_drawn_in_the_sequence_of_order() {
    let cols = columns_of(&input(vec![
        card("c", "todo", Some("2")),
        card("a", "todo", Some("0")),
        card("b", "todo", Some("1")),
        card("x", "doing", Some("10")),
        card("y", "doing", Some("9")),
    ]));
    assert_eq!(keys(&cols, 0), ["a", "b", "c"]);
    assert_eq!(keys(&cols, 1), ["y", "x"]);
}

#[test]
fn cards_without_a_number_stay_in_the_order_they_came_and_stand_behind_the_numbered_ones() {
    let cols = columns_of(&input(vec![
        card("n1", "todo", None),
        card("b", "todo", Some("5")),
        card("n2", "todo", Some("soon")),
        card("a", "todo", Some("5")),
        card("n3", "todo", Some("")),
    ]));
    // equal numbers keep the delivery (stable): b before a
    assert_eq!(keys(&cols, 0), ["b", "a", "n1", "n2", "n3"]);
}

#[test]
fn many_cards_with_the_same_number_keep_the_delivery_the_sort_is_stable() {
    let num = |i: usize| (i * 7 + i / 3) % 5;
    let rows: Vec<_> = (0..200)
        .map(|i| card(&format!("k{i:03}"), "todo", Some(&num(i).to_string())))
        .collect();
    let mut want: Vec<usize> = (0..200).collect();
    want.sort_by_key(|i| (num(*i), *i));
    let want: Vec<String> = want.iter().map(|i| format!("k{i:03}")).collect();
    assert_eq!(keys(&columns_of(&input(rows)), 0), want);
}

fn facts_input(rows: Vec<FormRow>) -> FormInput {
    let field = |n: &str| FieldInfo {
        name: n.into(),
        block: "field-text".into(),
    };
    let mut i = input(rows);
    i.channels = vec![ChannelInfo {
        name: "facts".into(),
        fields: vec![field("label"), field("due")],
        choices: Vec::new(),
    }];
    i
}

fn row_with(assignees: &[&str]) -> FormRow {
    let mut r = FormRow::of("a", &[("title", "T"), ("column", "c")]);
    r.values
        .push(("facts".into(), vec!["docs".into(), "Fri".into()]));
    r.values.push((
        "assignee".into(),
        assignees.iter().map(|s| (*s).to_owned()).collect(),
    ));
    r
}

#[test]
fn repeated_assignees_do_not_move_the_facts() {
    let cols = columns_of(&facts_input(vec![row_with(&["Ada Lovelace", "Bob"])]));
    let card = &cols[0].cards[0];
    assert_eq!(card.assignees, ["Ada Lovelace", "Bob"]);
    assert_eq!(card.assignee(), Some("Ada Lovelace"));
    let lines: Vec<String> = card.shown_facts().map(FieldSlot::line).collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].contains("docs") && lines[1].contains("Fri"),
        "{lines:?}"
    );
    // the same facts with no assignee at all
    let none = columns_of(&facts_input(vec![row_with(&[])]));
    assert_eq!(none[0].cards[0].assignees, Vec::<String>::new());
}

#[test]
fn a_repeated_fact_value_is_not_laid_on_a_wrong_label() {
    let mut r = row_with(&[]);
    r.values.retain(|(c, _)| c != "facts");
    r.values
        .push(("facts".into(), vec!["a".into(), "b".into(), "c".into()]));
    let cols = columns_of(&facts_input(vec![r]));
    let lines: Vec<String> = cols[0].cards[0]
        .shown_facts()
        .map(FieldSlot::line)
        .collect();
    assert_eq!(lines.len(), 3, "{lines:?}");
    let labels: Vec<&str> = cols[0].cards[0]
        .shown_facts()
        .map(|f| f.label.as_str())
        .collect();
    assert!(labels.iter().all(|l| *l == "fact.facts"), "{labels:?}");
}

fn who_input(also_in_assignee: bool) -> FormInput {
    let mut i = facts_input(Vec::new());
    let who = FieldInfo {
        name: "who".into(),
        block: "field-text".into(),
    };
    i.channels[0].fields = vec![i.channels[0].fields[0].clone(), who.clone()];
    if also_in_assignee {
        i.channels.push(ChannelInfo {
            name: "assignee".into(),
            fields: vec![who],
            choices: Vec::new(),
        });
    }
    let mut r = FormRow::of("a", &[("title", "T"), ("column", "c")]);
    r.values
        .push(("facts".into(), vec!["docs".into(), "Ada Lovelace".into()]));
    if also_in_assignee {
        r.values
            .push(("assignee".into(), vec!["Ada Lovelace".into()]));
    }
    i.rows = vec![r];
    i
}

#[test]
fn a_field_only_in_facts_stays_a_fact_whatever_its_name() {
    let cols = columns_of(&who_input(false));
    let lines: Vec<String> = cols[0].cards[0]
        .shown_facts()
        .map(FieldSlot::line)
        .collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(cols[0].cards[0].assignees, Vec::<String>::new());
}

#[test]
fn a_field_in_both_channels_shows_only_as_initials() {
    let cols = columns_of(&who_input(true));
    let card = &cols[0].cards[0];
    assert_eq!(card.shown_facts().count(), 1);
    assert_eq!(card.assignees, ["Ada Lovelace"]);
}

#[test]
fn two_assignees_show_two_initials_and_the_overflow_is_counted() {
    use crate::meta::card_tail;
    let two = columns_of(&facts_input(vec![row_with(&["Ada Lovelace", "Bob"])]));
    assert_eq!(card_tail(&two[0].cards[0]), ["AL", "B"]);
    let many = columns_of(&facts_input(vec![row_with(&[
        "A a", "B b", "C c", "D d", "E e",
    ])]));
    assert_eq!(card_tail(&many[0].cards[0]), ["AA", "BB", "CC", "+2"]);
}
