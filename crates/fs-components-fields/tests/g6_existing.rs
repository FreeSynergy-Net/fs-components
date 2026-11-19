//! Bundle G6: radio group and switch are NOT rebuilt; the existing `segmented` (role `radiogroup`, `radio`, Arrow keys) and `switch`
//! (role `switch`, Space/Enter) of this crate are the pieces, and the catalog names them.
//!
//! Tests: the contract text names the blocks; the roles are read from the built elements.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use common::{attr, layout, serial};
use fs_components_fields::controls::{Segmented, Switch};

const GENERAL: &str = include_str!("../../../contract/blocks/general.toml");
const FIELDS: &str = include_str!("../../../contract/blocks/fields.toml");
const DIALOGS: &str = include_str!("../../../contract/blocks/dialogs.toml");

#[test]
fn the_catalog_names_the_existing_blocks_for_radio_group_switch_and_badge() {
    for block in ["switch", "segmented", "badge", "tag"] {
        assert!(
            [GENERAL, FIELDS, DIALOGS]
                .iter()
                .any(|t| t.contains(&format!("[block.{block}]"))),
            "block {block} is in the catalog"
        );
    }
}

#[test]
fn the_existing_pieces_have_the_roles_of_the_apg() {
    let _g = serial();
    let sw = Switch::new("Timer").id("g6-sw");
    let tree = layout(sw.build(), 300.0, 60.0);
    assert_eq!(attr(&tree, "g6-sw", "role").as_deref(), Some("switch"));
    let sg = Segmented::new()
        .id("g6-sg")
        .segment("a", "A", false)
        .segment("b", "B", false);
    let tree = layout(sg.build(), 300.0, 60.0);
    assert_eq!(attr(&tree, "g6-sg", "role").as_deref(), Some("radiogroup"));
    assert_eq!(attr(&tree, "g6-sg-a", "role").as_deref(), Some("radio"));
}
