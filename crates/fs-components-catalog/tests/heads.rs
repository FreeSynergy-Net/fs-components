//! C4-0: the key `heads` of a grouping slot (the forms whose rows are the heads of the groups); the slot `lanes` of the board names
//! `rows@^1.0.0`.
//!
//! Mutant: the parser ignores `heads` -> `lanes_names_its_head_form` and `heads_is_a_known_word` fail.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
use fs_components_catalog::{AcceptsCode, BlockCatalog, read_blocks};
use fs_types::coded::Coded;

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const WIDGETS: &str = include_str!("../../../contract/blocks/widgets.toml");
const FRAMES: &str = include_str!("../../../contract/blocks/frames.toml");
const GENERAL: &str = include_str!("../../../contract/blocks/general.toml");
const PROGRAMS: &str = include_str!("../../../contract/blocks/programs.toml");

fn block(slot: &str) -> String {
    format!("[block.a]\naccepts = [{{ slot = \"r\", kind = \"records\" }}, {slot}]\n")
}

fn problems(text: &str) -> Vec<Coded<AcceptsCode>> {
    read_blocks("t.toml", text).expect_err("a problem is wanted")
}

#[test]
fn lanes_names_its_head_form() {
    let c = BlockCatalog::read(
        CONTRACT,
        &[
            ("frames.toml", FRAMES),
            ("general.toml", GENERAL),
            ("programs.toml", PROGRAMS),
            ("widgets.toml", WIDGETS),
        ],
    )
    .unwrap();
    let board = c.need("board").unwrap();
    let heads: Vec<String> = board
        .slot("lanes")
        .unwrap()
        .heads
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(heads, ["rows@^1.0.0"]);
    assert!(board.slot("columns").unwrap().heads.is_empty());
}

#[test]
fn heads_is_a_known_word() {
    let ok =
        block("{ slot = \"g\", kind = \"grouping\", within = \"r\", heads = [\"rows@^1.0.0\"] }");
    let got = read_blocks("t.toml", &ok).unwrap();
    assert_eq!(got[0].slot("g").unwrap().heads.len(), 1);
}

#[test]
fn a_wrong_heads_is_a_slot_problem() {
    let g = |extra: &str| {
        block(&format!(
            "{{ slot = \"g\", kind = \"grouping\", within = \"r\", {extra} }}"
        ))
    };
    for bad in [
        "heads = []",
        "heads = \"rows@^1.0.0\"",
        "heads = [\"rows\"]",
        "heads = [1]",
        "heads = [\"rows@^1.0.0\", \"rows@^1.1.0\"]",
    ] {
        assert_eq!(problems(&g(bad)).len(), 1, "{bad}");
    }
    // not a grouping slot
    let cmd = block("{ slot = \"c\", kind = \"command\", heads = [\"rows@^1.0.0\"] }");
    assert_eq!(problems(&cmd).len(), 1);
}
