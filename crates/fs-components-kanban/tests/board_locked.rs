//! Locked controls of the one board element (moved from `view_locked.rs`, ADR 0096 H-K5): a control of an operation the program does
//! not support is drawn locked and says "soon" (it is not left out), does nothing, and unlocks by itself when `supports(op)` turns true.
//!
//! Mutants (each makes the named test fail): an unsupported foot is left out again (`div().h(FOOT_H)`) ->
//! `unsupported_operations_are_visible_locked_and_say_soon_and_do_nothing`; the locked foot opens the entry on a click -> same
//! test; a live foot is drawn locked -> `supported_controls_ask_the_host_and_unlock_by_themselves`; the column button stays live
//! without any right -> `unsupported_operations_...`; the check reads a stale `supports` (the state is kept from the first build) ->
//! `supported_controls_ask_the_host_and_unlock_by_themselves`.
//!
//! Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, sample};
use fs_components_fields::Look;
use fs_components_kanban::{BoardBench, Labels, MemoryBoard, Operation};
use fs_components_styles::StyleId;

const SIZE: (f32, f32) = (1100.0, 640.0);

fn bench(b: &MemoryBoard, style: StyleId) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b.clone()))
        .look(Look::new(style, false))
        .id("B")
}

fn locked(b: MemoryBoard) -> MemoryBoard {
    [
        Operation::Move,
        Operation::AddCard,
        Operation::AddColumn,
        Operation::Rename,
        Operation::Limit,
    ]
    .into_iter()
    .fold(b, MemoryBoard::without)
    .without_menu()
}

#[test]
fn the_board_draws_columns_cards_marks_and_the_foot_in_every_style() {
    for style in StyleId::ALL {
        let b = sample();
        let rig = Rig::new(bench(&b, style), SIZE.0, SIZE.1);
        for id in [
            "B-head-todo",
            "B-head-doing",
            "B-head-done",
            "B-card-a",
            "B-card-d",
            "B-wip-todo",
            "B-add-todo",
            "B-menu-todo",
        ] {
            assert!(rig.has(id), "{style:?}: no {id}");
        }
        assert!(!rig.has("B-notice"), "{style:?}");
        assert!(!rig.has("B-soon-todo"), "{style:?}: nothing is locked");
    }
}

#[test]
fn unsupported_operations_are_visible_locked_and_say_soon_and_do_nothing() {
    let b = locked(sample());
    let mut rig = Rig::new(bench(&b, StyleId::Admin), SIZE.0, SIZE.1);
    for column in ["todo", "doing", "done"] {
        assert!(
            rig.has(&format!("B-add-{column}")),
            "the foot of {column} stays visible"
        );
        assert!(rig.has(&format!("B-soon-{column}")), "and says soon");
    }
    let soon = Labels::default().soon;
    let said = rig.tree.iter_nodes().any(|(_, n)| {
        matches!(&n.element_type,
            blinc_layout::renderer::ElementType::Text(t) if t.content == soon)
    });
    assert!(said, "the word is {soon:?}");
    let flag = |rig: &Rig, id: &str| {
        rig.tree
            .element_registry()
            .attribute(rig.tree.query_by_id(id).expect("element"), "data-locked")
    };
    assert_eq!(flag(&rig, "B-add-todo").as_deref(), Some("true"));
    assert_eq!(
        flag(&rig, "B-menu-todo").as_deref(),
        Some("true"),
        "the column button with no right is locked"
    );
    rig.click("B-add-todo");
    assert!(!rig.has("B-quick-todo"), "a locked foot opens no entry");
    rig.click("B-menu-todo");
    rig.click("B-card-a");
    rig.drag_to("B-card-a", rig.centre("B-head-done"));
    assert!(
        b.log()
            .iter()
            .all(|l| !l.starts_with("moved") && !l.starts_with("new")),
        "locked controls do nothing: {:?}",
        b.log()
    );
    let todo: Vec<_> = b.snapshot()[0]
        .cards
        .iter()
        .map(|c| c.key.clone())
        .collect();
    assert_eq!(todo, ["a", "b", "c"], "a locked move moves nothing");
}

#[test]
fn supported_controls_ask_the_host_and_unlock_by_themselves() {
    let b = locked(sample());
    let mut rig = Rig::new(bench(&b, StyleId::Windows), SIZE.0, SIZE.1);
    assert!(rig.has("B-soon-todo"));
    rig.click("B-add-todo");
    assert!(!rig.has("B-quick-todo"), "locked: nothing opens");
    b.supporting(Operation::AddCard);
    rig.rebuild();
    assert!(
        !rig.has("B-soon-todo"),
        "supported: no lock, without a new build"
    );
    rig.click("B-add-todo");
    assert!(rig.has("B-quick-todo"), "the live foot opens its entry");
}
