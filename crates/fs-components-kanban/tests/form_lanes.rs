//! The lanes from the form: the heads of the host (`FormInput::heads`) reach the board through `FormSource::lane_plan`. None = plain
//! columns, `Missing` = plain columns and the line that says why, `Rows` = a band per head in the host's order, a card in the lane of
//! each of its `assignee` values, cards without a matching value in the last band; `set` takes the heads of a new reading.
//!
//! Mutants: heads ignored (`two_heads_are_two_bands_in_order`); only the first assignee counts (`a_card_with_two_assignees_stands_in_two_lanes`);
//! `set` keeps the old heads (`reading_again_turns_lanes_on`).
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

mod board_common;
#[macro_use]
mod common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_core::form_read::{ChannelInfo, FormInput, FormRow, HeadRow, Heads};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_core::states::Load;
use fs_components_fields::KeysAsText;
use fs_components_kanban::{BoardBench, BoardSource, FormSource, Labels, LanePlan};

struct Host;

impl FormWrite for Host {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        false
    }
    gestures_by_channel!();

    fn send(&self, _: &str, _: Option<&str>, _: &[WriteArg]) -> Result<WriteDone, WriteRefused> {
        Ok(WriteDone::default())
    }
}

fn row(key: &str, who: &[&str]) -> FormRow {
    let who: Vec<(&str, &str)> = who.iter().map(|w| ("who", *w)).collect();
    FormRow::of(key, &[("title", key), ("column", "todo")]).with_values("assignee", &who)
}

fn head(key: &str, label: &str) -> HeadRow {
    HeadRow {
        key: key.into(),
        label: label.into(),
    }
}

fn input(heads: Option<Heads>) -> FormInput {
    FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![
            row("a", &["ada"]),
            row("b", &["ada", "bob"]),
            row("n", &["nobody-we-know"]),
            row("m", &[]),
        ],
        channels: vec![ChannelInfo {
            name: "column".into(),
            fields: Vec::new(),
            choices: vec![("todo".into(), "To do".into())],
        }],
        may: Vec::new(),
        load: Load::Ready,
        heads,
    }
}

fn two() -> Option<Heads> {
    Some(Heads::Rows(vec![head("bob", "Bob"), head("ada", "")]))
}

fn rig(heads: Option<Heads>) -> (Rig, Arc<FormSource>, BoardBench) {
    let source = Arc::new(FormSource::new(
        input(heads),
        Arc::new(Host),
        Arc::new(KeysAsText),
    ));
    let bench = BoardBench::new(source.clone(), Labels::default()).id("B");
    (Rig::new(bench.clone(), 1100.0, 900.0), source, bench)
}

#[test]
fn no_heads_asked_is_the_plain_board() {
    let (rig, source, _) = rig(None);
    assert_eq!(source.lane_plan(), LanePlan::Off);
    assert!(rig.has("B-card-a") && !rig.has("B-lane-ada-band") && !rig.has("B-lanes-missing"));
}

#[test]
fn asked_but_not_come_is_the_plain_board_with_the_line() {
    let (rig, source, _) = rig(Some(Heads::Missing));
    assert_eq!(source.lane_plan(), LanePlan::Missing);
    assert!(rig.has("B-card-a") && rig.has("B-lanes-missing"));
}

#[test]
fn two_heads_are_two_bands_in_order() {
    let (rig, source, _) = rig(two());
    match source.lane_plan() {
        LanePlan::On { grouping, heads } => {
            assert_eq!(grouping, "assignee");
            let keys: Vec<_> = heads
                .iter()
                .map(|h| (h.key.as_str(), h.label.as_str()))
                .collect();
            assert_eq!(
                keys,
                [("bob", "Bob"), ("ada", "ada")],
                "empty label shows the key"
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(rig.has("B-lane-ada-band") && rig.has("B-lane-bob-band"));
    let (_, ybob, _, _) = rig.bounds("B-lane-bob-band");
    let (_, yada, _, _) = rig.bounds("B-lane-ada-band");
    assert!(ybob < yada, "bands in the order of the heads");
}

#[test]
fn a_card_with_two_assignees_stands_in_two_lanes() {
    let (rig, _, _) = rig(two());
    assert!(rig.has("B-lane-ada-card-b") && rig.has("B-lane-bob-card-b"));
    assert!(rig.has("B-lane-ada-card-a") && !rig.has("B-lane-bob-card-a"));
}

#[test]
fn a_card_without_a_matching_value_is_in_the_last_band() {
    let (rig, _, _) = rig(two());
    assert!(rig.has("B-lane--none-card-m"), "no assignee");
    // a value no head names gets a lane of its own at the end (nothing hidden): "n" is not lost
    assert!(rig.has("B-lane-nobody-we-know-card-n"));
}

#[test]
fn reading_again_turns_lanes_on() {
    let (mut rig, source, bench) = rig(None);
    assert!(!rig.has("B-lane-ada-band"));
    let before = source.generation();
    source.set(input(two()));
    assert!(source.generation() > before);
    assert!(bench.sync(), "the board sees the new reading");
    rig.rebuild();
    assert!(rig.has("B-lane-ada-band") && rig.has("B-lane-bob-band"));
}
