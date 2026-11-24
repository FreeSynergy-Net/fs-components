//! The key `widget` of a block: the host learns from the catalog which block has a component (SB-29), read from the real files.
use fs_components_catalog::BlockCatalog;

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const GENERAL: &str = include_str!("../../../contract/blocks/general.toml");
const FRAMES: &str = include_str!("../../../contract/blocks/frames.toml");
const WIDGETS: &str = include_str!("../../../contract/blocks/widgets.toml");

fn real() -> BlockCatalog {
    BlockCatalog::read(
        CONTRACT,
        &[
            ("frames.toml", FRAMES),
            ("general.toml", GENERAL),
            ("widgets.toml", WIDGETS),
        ],
    )
    .expect("the contract files read")
}

fn widget_of(c: &BlockCatalog, block: &str) -> Option<String> {
    c.block(block).expect("block").widget.clone()
}

#[test]
fn a_block_with_a_widget_names_it_and_a_frame_names_none() {
    let c = real();
    assert_eq!(widget_of(&c, "field-card").as_deref(), Some("FieldCard"));
    assert_eq!(widget_of(&c, "contact-row").as_deref(), Some("PeopleWidget"));
    assert_eq!(widget_of(&c, "board").as_deref(), Some("fs-components-kanban"));
    assert_eq!(widget_of(&c, "app-window"), None);
    assert_eq!(widget_of(&c, "workbench"), None);
}
