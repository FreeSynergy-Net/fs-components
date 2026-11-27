//! A card carried from a lane into another lane: the request names both lanes (`from_lane`, `to_lane`); within one column the lane
//! alone is a move too; a place that refuses says why at the ghost; a closed lane takes nothing; the pictures (`FS_COMP_SHOTS=<dir>`).
//!
//! Tests: input tests over the real layout (the card component carries, the places are measured). Mutants: the lane is not sent
//! (`a_card_carried_into_another_lane_names_both_lanes`); a move inside a column is dropped as "nothing changed"
//! (`the_lane_alone_is_a_move`). Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::{Arc, Mutex};

use board_common::{Rig, Spot, carry, hold, serial};
use fs_components_fields::Look;
use fs_components_kanban::{
    Answer, BoardBench, BoardEvents, Card, Column, Labels, LaneHead, LanePlan, MemoryBoard,
    MoveRequest, Reason,
};
use fs_components_styles::StyleId;

#[derive(Default)]
struct Recorder(Mutex<Vec<MoveRequest>>);

impl BoardEvents for Recorder {
    fn moved(&self, r: &MoveRequest) -> Answer {
        self.0.lock().unwrap().push(r.clone());
        Answer::Accepted
    }
}

fn card(key: &str, who: &str) -> Card {
    let mut c = Card::new(key, &format!("Card {key}"));
    c.assignees = vec![who.into()];
    c
}

fn board() -> MemoryBoard {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![card("a", "ada"), card("b", "ada"), card("c", "bob")];
    let doing = Column::new("doing", "Doing");
    let mut done = Column::new("done", "Done");
    done.cards = vec![card("d", "bob")];
    MemoryBoard::new(vec![todo, doing, done]).with_lanes(LanePlan::On {
        grouping: "assignee".into(),
        heads: vec![LaneHead::new("ada", "Ada"), LaneHead::new("bob", "Bob")],
    })
}

fn bench(b: &MemoryBoard, events: Arc<dyn BoardEvents>, style: StyleId) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(events)
        .look(Look::new(style, false))
        .id("B")
}

#[test]
fn a_card_carried_into_another_lane_names_both_lanes() {
    let rec = Arc::new(Recorder::default());
    let bench = bench(&board(), rec.clone(), StyleId::Admin);
    carry(&bench, "ada/a", "B-lane-bob-col-doing", Spot::End);
    let seen = rec.0.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "{seen:?}");
    let r = &seen[0];
    assert_eq!(
        (r.card.as_str(), r.from.as_str(), r.to.as_str()),
        ("a", "todo", "doing")
    );
    assert_eq!(
        (r.from_lane.as_deref(), r.to_lane.as_deref()),
        (Some("ada"), Some("bob"))
    );
}

#[test]
fn the_lane_alone_is_a_move() {
    // c stands in the lane of bob in "todo"; carried to the lane of ada in the same column it is a request: same column, other lane
    let rec = Arc::new(Recorder::default());
    let bench = bench(&board(), rec.clone(), StyleId::Admin);
    let mut rig = Rig::new(bench, 1500.0, 900.0);
    board_common::carry_in(
        &mut rig,
        "B-lane-bob-card-c",
        "B-lane-ada-col-todo",
        Spot::After("ada/b"),
    );
    let seen = rec.0.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert_eq!(
        (seen[0].from.as_str(), seen[0].to.as_str()),
        ("todo", "todo")
    );
    assert_eq!(
        (seen[0].from_lane.as_deref(), seen[0].to_lane.as_deref()),
        (Some("bob"), Some("ada"))
    );
    assert_eq!(
        seen[0].after.as_deref(),
        Some("b"),
        "right under b in the lane of ada"
    );
}

#[test]
fn a_place_that_refuses_gets_no_request_and_says_why() {
    let rec = Arc::new(Recorder::default());
    let b = board().refusing("done", Reason::new("no-right", "You may not finish cards."));
    let bench = bench(&b, rec.clone(), StyleId::Admin);
    carry(&bench, "ada/a", "B-lane-ada-col-done", Spot::End);
    assert!(rec.0.lock().unwrap().is_empty());
    assert_eq!(
        bench.read(|s| s.notice().map(|n| (n.text.clone(), n.refused))),
        Some(("You may not finish cards.".into(), true))
    );
}

#[test]
fn a_closed_lane_takes_nothing_and_its_cards_are_folded_away() {
    let mut rig = Rig::new(
        bench(&board(), Arc::new(Recorder::default()), StyleId::Admin),
        1500.0,
        900.0,
    );
    rig.click("B-grid-cell-bob-todo");
    rig.rebuild();
    assert!(rig.has("B-grid-row-bob") && !rig.has("B-lane-bob-card-c"));
    assert!(
        !rig.has("B-lane-bob-col-doing"),
        "no place to put a card down in"
    );
    assert!(rig.has("B-lane-ada-card-a"));
}

// ---- pictures (FS_COMP_SHOTS=<dir>) in the style and the appearance asked for, with the theme guard of the widgets' `styled.rs` ----

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;
#[path = "../../fs-components-widgets/tests/common/styled.rs"]
mod styled;

fn registry(tree: &blinc_layout::prelude::RenderTree) {
    blinc_core::BlincContextState::get().set_element_registry(
        Arc::clone(tree.element_registry()) as blinc_core::AnyElementRegistry,
    );
}

/// Two passes (as the card pictures do): the first lays the board out, the second draws the layer from what the first measured.
fn shoot(b: &BoardBench, name: &str, style: StyleId, dark: bool) {
    let _g = serial();
    if std::env::var_os("FS_COMP_SHOTS").is_none() {
        return;
    }
    let css = fs_components_kanban::styles();
    let make = |b: &BoardBench| {
        let b = b.clone();
        move || b.build(1000.0, 640.0)
    };
    let first = styled::layout_styled(make(b), 1000.0, 640.0, style, dark, &css);
    registry(&first);
    let tree = styled::layout_styled(make(b), 1000.0, 640.0, style, dark, &css);
    registry(&tree);
    styled::draw_styled_tree(&tree, 1000, 640, style, dark, &format!("{name}.png"));
}

#[test]
fn pictures_of_the_lanes_and_the_carry() {
    for style in [StyleId::Admin, StyleId::Simple] {
        for dark in [false, true] {
            let rec: Arc<dyn BoardEvents> = Arc::new(Recorder::default());
            shoot(&bench(&board(), rec.clone(), style), "lanes", style, dark);
            let folded = bench(&board(), rec.clone(), style);
            {
                let mut rig = Rig::new(folded.clone(), 1000.0, 640.0);
                rig.click("B-grid-cell-bob-todo");
            }
            shoot(&folded, "lane-closed", style, dark);
            let carried = bench(&board(), rec.clone(), style);
            hold(&carried, "ada/a", "B-lane-bob-col-doing", Spot::End);
            shoot(&carried, "carry-foreign-lane", style, dark);
            let b = board().refusing("done", Reason::new("no-right", "You may not finish cards."));
            let refused = bench(&b, rec.clone(), style);
            hold(&refused, "ada/a", "B-lane-ada-col-done", Spot::End);
            shoot(&refused, "carry-refused", style, dark);
        }
    }
}
