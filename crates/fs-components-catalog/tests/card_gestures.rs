//! The gestures of the blocks as command slots (P1 of the board-in-grid draft, rule M1; 0.5.0: `card` has `activate` and `menu`, the
//! containers `drop` and `quick-add` with the args of their form): the slot name is the trigger of `[[action]]`, `args` is the stable
//! argument schema. Read from the real contract files; the key `args` is closed (command only).
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

fn args_of(c: &BlockCatalog, block: &str, slot: &str) -> Vec<String> {
    c.block(block)
        .unwrap()
        .slot(slot)
        .expect("slot")
        .args
        .clone()
}

#[test]
fn card_has_the_two_gesture_slots_as_commands_and_no_drop() {
    let c = real();
    let card = c.block("card").unwrap();
    let names: Vec<&str> = card.accepts.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["activate", "menu"]);
    assert!(card.accepts.iter().all(|s| s.kind == SlotKind::Command));
    assert_eq!(card.slot("menu").unwrap().repeat, 8);
}

#[test]
fn drop_is_at_the_container_with_the_arguments_of_its_form_in_order() {
    let c = real();
    assert_eq!(
        args_of(&c, "board", "drop"),
        ["column", "lane", "after", "before"]
    );
    assert_eq!(
        args_of(&c, "board", "quick-add"),
        ["title", "column", "lane"]
    );
    assert_eq!(
        args_of(&c, "calendar-grid", "drop"),
        ["start", "end", "lane"]
    );
    assert_eq!(
        args_of(&c, "calendar-grid", "quick-add"),
        ["label", "start", "end", "all-day"]
    );
}

#[test]
fn gestures_without_args_have_none() {
    let c = real();
    for block in ["card", "board", "calendar-grid"] {
        for slot in ["activate", "menu"] {
            assert_eq!(args_of(&c, block, slot), NONE, "{block}.{slot}");
        }
    }
    assert_eq!(c.block("board").unwrap().slot("menu").unwrap().repeat, 8);
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
        "[block.t]\naccepts = [{ slot = \"drop\", kind = \"command\", args = [\"a-b\", \"c\"] }]\n";
    assert_eq!(
        read_blocks("t.toml", ok).unwrap()[0].accepts[0].args,
        ["a-b", "c"]
    );
    let none = "[block.t]\naccepts = [{ slot = \"drop\", kind = \"command\", args = [] }]\n";
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
            "{{ slot = \"drop\", kind = \"command\", args = {bad} }}"
        ));
        assert_eq!(f, ["args"], "{bad}");
    }
}

#[test]
fn record_rows_have_activate_and_menu_so_edit_and_delete_hang_at_the_line() {
    // User rule 27.11.: every record type can create, edit and delete; the row gets the same two slots as `card`.
    let c = real();
    for block in ["contact-row", "task-row", "program-row", "page-tree-row"] {
        let b = c.block(block).unwrap();
        let names: Vec<&str> = b.accepts.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["activate", "menu"], "{block}");
        assert!(
            b.accepts.iter().all(|s| s.kind == SlotKind::Command),
            "{block}"
        );
        assert_eq!(b.slot("menu").unwrap().repeat, 8, "{block}");
        assert_eq!(args_of(&c, block, "activate"), NONE, "{block}");
        assert_eq!(args_of(&c, block, "menu"), NONE, "{block}");
    }
}

#[test]
fn revision_row_is_read_only_history_and_has_no_command_slot() {
    assert_eq!(real().block("revision-row").unwrap().accepts.len(), 0);
}
