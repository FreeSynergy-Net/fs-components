//! The names of the command slots (contract 0.5.0): a trigger word, or `<part>-<trigger word>` with a part of the block; never a
//! command name (`move`, `card-menu`) or a channel of a form. Read from the real contract files and from small texts.
//!
//! Pattern: none (tests).
mod support;

use fs_components_catalog::{
    AcceptsCode, BlockCatalog, SlotKind, TRIGGERS, is_gesture_slot, read_blocks,
};
use support::util::*;

const CONTRACT_TOML: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 6] = [
    (
        "dialogs.toml",
        include_str!("../../../contract/blocks/dialogs.toml"),
    ),
    (
        "fields.toml",
        include_str!("../../../contract/blocks/fields.toml"),
    ),
    (
        "frames.toml",
        include_str!("../../../contract/blocks/frames.toml"),
    ),
    (
        "general.toml",
        include_str!("../../../contract/blocks/general.toml"),
    ),
    (
        "programs.toml",
        include_str!("../../../contract/blocks/programs.toml"),
    ),
    (
        "widgets.toml",
        include_str!("../../../contract/blocks/widgets.toml"),
    ),
];

fn block_with(slot: &str, parts: &str) -> String {
    format!("[block.t]\naccepts = [{slot}]\nparts = {parts}\n")
}

#[test]
fn the_word_list_is_the_four_triggers() {
    assert_eq!(TRIGGERS, ["activate", "menu", "drop", "quick-add"]);
}

#[test]
fn every_command_slot_of_the_real_contract_is_a_trigger_word() {
    let cat = BlockCatalog::read(CONTRACT_TOML, &FILES).expect("the contract reads");
    let mut seen = 0;
    for block in cat.iter() {
        for slot in block.accepts.iter().filter(|s| s.kind == SlotKind::Command) {
            seen += 1;
            assert!(
                TRIGGERS.contains(&slot.name.as_str()),
                "{}.{}",
                block.name,
                slot.name
            );
        }
    }
    assert!(seen >= 12, "{seen} command slots");
}

#[test]
fn the_rule_takes_trigger_words_and_part_gestures() {
    let parts = ["column", "root"];
    for ok in [
        "activate",
        "menu",
        "drop",
        "quick-add",
        "column-menu",
        "root-quick-add",
    ] {
        assert!(is_gesture_slot(ok, &parts), "{ok}");
    }
    for bad in [
        "move",
        "card-menu",
        "entries",
        "actions",
        "column-move",
        "-menu",
        "menu-",
        "columnmenu",
        "drops",
    ] {
        assert!(!is_gesture_slot(bad, &parts), "{bad}");
    }
}

#[test]
fn a_command_slot_with_another_name_is_refused_with_the_field_slot() {
    for bad in ["move", "card-menu", "entries"] {
        let text = block_with(
            &format!("{{ slot = \"{bad}\", kind = \"command\" }}"),
            "[\"root\"]",
        );
        let e = read_blocks("t.toml", &text).unwrap_err();
        assert_eq!(
            show(&e),
            [format!("SlotInvalid[block=t,slot={bad},field=slot]")]
        );
        assert_eq!(codes(&e), [AcceptsCode::SlotInvalid]);
    }
}

#[test]
fn a_part_gesture_needs_the_part_in_the_block() {
    let slot = "{ slot = \"column-menu\", kind = \"command\" }";
    assert!(read_blocks("t.toml", &block_with(slot, "[\"root\", \"column\"]")).is_ok());
    assert!(read_blocks("t.toml", &block_with(slot, "[\"root\"]")).is_err());
    assert!(read_blocks("t.toml", &format!("[block.t]\naccepts = [{slot}]\n")).is_err());
}

#[test]
fn only_command_slots_are_named_by_the_rule() {
    // a data slot keeps its free kebab-case name
    let text = block_with(
        "{ slot = \"card-title\", kind = \"records\" }",
        "[\"root\"]",
    );
    assert!(read_blocks("t.toml", &text).is_ok());
}
