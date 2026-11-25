//! The board in lanes: a band per lane with its head line (name, count, fold), the columns under it; empty and headless lanes drawn,
//! the cards without a lane in a last band; without a list of lanes the plain board; "list missing" said when it was asked for.
//!
//! Tests: input tests over Blinc's tree and router. Mutants: the lanes are not drawn (`a_lane_is_a_band_with_its_cards`); empty lanes left out
//! (`an_empty_lane_and_a_lane_without_head_are_drawn`); a fold that hides the head too (`the_fold_hides_the_columns_not_the_head`).
//! Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::Rig;
use fs_components_kanban::{BoardBench, Card, Column, Labels, LaneHead, LanePlan, MemoryBoard};

const SIZE: (f32, f32) = (1100.0, 700.0);

fn card(key: &str, who: &[&str]) -> Card {
    let mut c = Card::new(key, &format!("Card {key}"));
    c.assignees = who.iter().map(|w| (*w).to_owned()).collect();
    c
}

fn host(plan: LanePlan) -> MemoryBoard {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![
        card("a", &["ada"]),
        card("b", &["ada", "carl"]),
        card("n", &[]),
    ];
    let mut done = Column::new("done", "Done");
    done.cards = vec![card("c", &["carl"])];
    MemoryBoard::new(vec![todo, done]).with_lanes(plan)
}

pub fn on() -> LanePlan {
    LanePlan::On {
        grouping: "assignee".into(),
        heads: vec![
            LaneHead::new("ada", "Ada Lovelace"),
            LaneHead::new("bob", "Bob Empty"),
        ],
    }
}

fn rig(plan: LanePlan) -> Rig {
    let b = host(plan);
    let bench = BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b))
        .id("B");
    Rig::new(bench, SIZE.0, SIZE.1)
}

#[test]
fn a_lane_is_a_band_with_its_cards() {
    let rig = rig(on());
    assert!(rig.has("B-lane-ada-band"));
    // the card of two lanes stands in both, with an id of its own in each
    assert!(rig.has("B-lane-ada-card-a") && rig.has("B-lane-ada-card-b"));
    assert!(rig.has("B-lane-carl-card-b") && rig.has("B-lane-carl-card-c"));
    assert!(!rig.has("B-card-a"), "no plain card next to the lane cards");
    // the heads of the columns once, on top
    assert!(rig.has("B-head-todo") && rig.has("B-head-done"));
}

#[test]
fn an_empty_lane_and_a_lane_without_head_are_drawn() {
    let rig = rig(on());
    assert!(
        rig.has("B-lane-bob-band"),
        "a lane the host named, with no card"
    );
    assert!(rig.has("B-lane-bob-empty"), "and it says so");
    assert!(rig.has("B-lane-carl-band"), "a lane only the cards name");
    assert!(
        rig.has("B-lane--none-band"),
        "cards in no lane are not lost"
    );
    assert!(rig.has("B-lane--none-card-n"));
}

#[test]
fn the_fold_hides_the_columns_not_the_head() {
    let mut rig = rig(on());
    assert!(rig.has("B-lane-ada-card-a"));
    rig.click("B-lane-ada");
    rig.rebuild();
    assert!(rig.has("B-lane-ada-band"), "the band stays");
    assert!(!rig.has("B-lane-ada-card-a"), "its cards are folded away");
    assert!(
        rig.has("B-lane-carl-card-c"),
        "the other lanes are untouched"
    );
    rig.click("B-lane-ada");
    rig.rebuild();
    assert!(rig.has("B-lane-ada-card-a"), "and back");
}

#[test]
fn without_lanes_the_board_is_what_it_was() {
    let rig = rig(LanePlan::Off);
    assert!(rig.has("B-card-a") && rig.has("B-col-todo"));
    assert!(!rig.has("B-lane-ada-band") && !rig.has("B-lanes-missing"));
}

#[test]
fn a_missing_list_falls_back_and_says_so() {
    let rig = rig(LanePlan::Missing);
    assert!(
        rig.has("B-card-a") && rig.has("B-col-todo"),
        "the plain board"
    );
    assert!(rig.has("B-lanes-missing"), "with the line that says why");
}
