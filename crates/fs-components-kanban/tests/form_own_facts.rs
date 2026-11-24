//! The channels `priority` and `estimated` of the form `cards-in-columns` (1.3.0) are facts of the card (decision C2: shown as a fact):
//! labelled `fact.priority` / `fact.estimated`, the duration drawn by the block `field-duration`. No value, no fact.
//!
//! Pattern: Adapter (rows in, cards out). Mutant: `card_of` reads only `facts` -> the first test fails.
#![allow(clippy::pedantic)]

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_kanban::columns_of;

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "cards-in-columns@1.3.0".into(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
    }
}

fn row(key: &str, extra: &[(&str, &str)]) -> FormRow {
    let mut r = FormRow::of(key, &[("title", key), ("column", "todo")]);
    for (k, v) in extra {
        r.values.push(((*k).into(), vec![(*v).into()]));
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
