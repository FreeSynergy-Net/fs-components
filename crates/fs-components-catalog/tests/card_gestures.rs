//! The gestures of the block `card` as command slots (P1 of the board-in-grid draft, rule M1): the slot name is the trigger of
//! `[[action]]`, `args` is the stable argument schema. Read from the real contract files; the key `args` is closed (command only).
//!
//! Pattern: none (tests).
mod support;

use fs_components_catalog::{AcceptsCode, BlockCatalog, SlotKind, read_blocks};
use support::util::*;

const CONTRACT_TOML: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 4] = [
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

const NONE: [&str; 0] = [];

fn real() -> BlockCatalog {
    BlockCatalog::read(CONTRACT_TOML, &FILES).expect("the contract files read")
}

fn args_of(c: &BlockCatalog, slot: &str) -> Vec<String> {
    c.block("card")
        .unwrap()
        .slot(slot)
        .expect("slot")
        .args
        .clone()
}

#[test]
fn card_has_the_three_gesture_slots_as_commands() {
    let c = real();
    let card = c.block("card").unwrap();
    let names: Vec<&str> = card.accepts.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["activate", "menu", "drop"]);
    assert!(card.accepts.iter().all(|s| s.kind == SlotKind::Command));
}

#[test]
fn drop_carries_the_arguments_of_the_form_cards_in_columns_in_order() {
    let c = real();
    assert_eq!(args_of(&c, "drop"), ["column", "lane", "after", "before"]);
    assert_eq!(args_of(&c, "activate"), NONE);
    assert_eq!(args_of(&c, "menu"), NONE);
}

#[test]
fn a_command_without_args_reads_with_none() {
    let c = real();
    // the board's own commands keep working unchanged
    for slot in ["move", "quick-add", "card-menu"] {
        assert_eq!(c.block("board").unwrap().slot(slot).unwrap().args, NONE);
    }
}

fn fields(slot: &str) -> Vec<String> {
    let text = format!(
        "[block.t]\naccepts = [\n  {{ slot = \"r\", kind = \"records\" }},\n  {slot},\n]\n"
    );
    let list = read_blocks("t.toml", &text).expect_err("must be refused");
    list.iter()
        .map(|c| {
            assert_eq!(c.code(), AcceptsCode::SlotInvalid);
            param(c, "field").unwrap_or_default()
        })
        .collect()
}

#[test]
fn args_are_closed_command_only_kebab_and_unique() {
    // good
    let ok =
        "[block.t]\naccepts = [{ slot = \"d\", kind = \"command\", args = [\"a-b\", \"c\"] }]\n";
    assert_eq!(
        read_blocks("t.toml", ok).unwrap()[0].accepts[0].args,
        ["a-b", "c"]
    );
    let none = "[block.t]\naccepts = [{ slot = \"d\", kind = \"command\", args = [] }]\n";
    assert_eq!(
        read_blocks("t.toml", none).unwrap()[0].accepts[0].args,
        NONE
    );
    // wrong kind
    assert_eq!(
        fields("{ slot = \"x\", kind = \"records\", args = [\"a\"] }"),
        ["args"]
    );
    // not a list, not text, bad word, twice
    for bad in ["\"a\"", "[1]", "[\"A\"]", "[\"a--b\"]", "[\"a\", \"a\"]"] {
        let f = fields(&format!(
            "{{ slot = \"x\", kind = \"command\", args = {bad} }}"
        ));
        assert_eq!(f, ["args"], "{bad}");
    }
}
