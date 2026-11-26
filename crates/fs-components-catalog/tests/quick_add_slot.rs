//! The block `quick-add` (the row at the end of a list) has its command slot `quick-add` with the argument `title` (contract 0.5.0),
//! written like the slot of the containers. Read from the real contract files.
//!
//! Pattern: none (tests).
use fs_components_catalog::{BlockCatalog, SlotKind, is_gesture_slot};

const CONTRACT_TOML: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 1] = [(
    "programs.toml",
    include_str!("../../../contract/blocks/programs.toml"),
)];

#[test]
fn the_row_has_the_command_slot_quick_add_with_the_argument_title() {
    let cat = BlockCatalog::read(CONTRACT_TOML, &FILES).expect("the contract reads");
    let block = cat.block("quick-add").expect("the block");
    assert_eq!(block.accepts.len(), 1);
    let slot = block.slot("quick-add").expect("the slot");
    assert_eq!(slot.kind, SlotKind::Command);
    assert_eq!(slot.args, ["title"]);
    assert!(is_gesture_slot(&slot.name, &[]));
}
