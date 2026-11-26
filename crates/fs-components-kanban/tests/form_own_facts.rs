//! The channels `priority` and `estimated` of the form `cards-in-columns` (1.3.0) are facts of the card (decision C2: shown as a fact):
//! labelled `fact.priority` / `fact.estimated`, the duration drawn by the block `field-duration`. No value, no fact.
//!
//! Pattern: Adapter (rows in, cards out). Mutant: `card_of` reads only `facts` -> the first test fails.
#![allow(clippy::pedantic)]

use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_kanban::columns_of;

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "cards-in-columns@1.3.0".into(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn row(key: &str, extra: &[(&str, &str)]) -> FormRow {
    let mut r = FormRow::of(key, &[("title", key), ("column", "todo")]);
    for (k, v) in extra {
        r = r.with_values(k, &[(k, v)]);
    }
    r
}

#[test]
fn priority_and_estimated_are_facts_of_the_card() {
    let cols = columns_of(&input(vec![row(
        "a",
        &[("priority", "2"), ("estimated", "PT1H")],
    )]));
    let facts: Vec<_> = cols[0].cards[0]
        .facts()
        .iter()
        .map(|f| (f.block.as_str(), f.label.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        [
            ("field-integer", "fact.priority", "2"),
            ("field-duration", "fact.estimated", "PT1H")
        ]
    );
}

#[test]
fn a_missing_or_empty_channel_adds_no_fact() {
    let cols = columns_of(&input(vec![
        row("none", &[]),
        row("empty", &[("priority", ""), ("estimated", "  ")]),
        row("one", &[("estimated", "PT30M")]),
    ]));
    assert!(cols[0].cards[0].facts().is_empty());
    assert!(cols[0].cards[1].facts().is_empty());
    assert_eq!(cols[0].cards[2].facts().len(), 1);
}

fn with_facts(names: &[&str]) -> FormInput {
    let mut i = input(Vec::new());
    i.channels.push(ChannelInfo {
        name: "facts".into(),
        fields: names
            .iter()
            .map(|n| FieldInfo {
                name: (*n).into(),
                block: "field-text".into(),
                label: String::new(),
            })
            .collect(),
        choices: Vec::new(),
    });
    i
}

#[test]
fn a_field_of_facts_with_the_same_name_is_not_drawn_twice() {
    let mut i = with_facts(&["priority"]);
    let r = row("a", &[("priority", "2"), ("estimated", "PT1H")])
        .with_values("facts", &[("priority", "2")]);
    i.rows.push(r);
    let cols = columns_of(&i);
    let labels: Vec<_> = cols[0].cards[0]
        .facts()
        .iter()
        .map(|f| f.label.as_str())
        .collect();
    assert_eq!(
        labels,
        ["fact.estimated", "fact.priority"],
        "once each: the own channel `priority` gives way to the field of `facts`"
    );
    assert_eq!(
        cols[0].cards[0].facts()[1].block,
        "field-text",
        "the mapped field wins"
    );
}

#[test]
fn priority_and_estimated_are_drawn_even_when_facts_is_full() {
    let mut i = with_facts(&["a", "b", "c", "d"]);
    let r = row("a", &[("priority", "2"), ("estimated", "PT1H")])
        .with_values("facts", &[("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]);
    i.rows.push(r);
    let card = &columns_of(&i)[0].cards[0];
    let labels: Vec<_> = card.facts().iter().map(|f| f.label.as_str()).collect();
    assert_eq!(
        labels,
        ["fact.priority", "fact.estimated", "fact.a", "fact.b"],
        "the own channels first, `facts` takes the rest of the limit"
    );
    assert_eq!(
        card.more_facts(),
        2,
        "the last two fields of `facts` say '+ 2 more'"
    );
}
