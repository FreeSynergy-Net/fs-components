//! The users of `disabled`: a disabled toolbar item and a locked command line keep their size, carry no extra text, and say why in a
//! tooltip that the keyboard reaches (ENTWURF-unfertig-tooltip U3, counter-checks G1 to G4). The menu has its own in `fs-components-app`.

mod common;

use common::{Input, attr, frame, layout_with, serial, size_of};
use fs_components_commands::{State, TextKey};
use fs_components_widgets::disabled::Why;
use fs_components_widgets::toolbar::{Item, STYLES, Toolbar};
use fs_components_widgets::why_from_state;

fn show(item: Item) -> blinc_layout::prelude::RenderTree {
    let bar = Toolbar::new(vec![Item::button("a", "Cut"), item]).id("tb");
    layout_with(bar.build(), 400.0, 120.0, STYLES)
}

#[test]
fn a_disabled_toolbar_item_has_the_size_of_a_free_one_and_no_extra_text() {
    let _g = serial();
    let free = show(Item::button("b", "Copy"));
    let off = show(Item::button("b", "Copy").disabled_because(Why::Unsupported));
    assert_eq!(
        size_of(&off, "tb-item-b"),
        size_of(&free, "tb-item-b"),
        "same size"
    );
    assert!(off.query_by_id("tb-item-b-why-bubble").is_none());
}

#[test]
fn a_disabled_toolbar_item_is_aria_disabled_and_the_keyboard_reads_the_reason() {
    let _g = serial();
    let mut tree =
        show(Item::button("b", "Copy").disabled_because(Why::Now("Nothing selected".into())));
    assert_eq!(
        attr(&tree, "tb-item-b", "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(attr(&tree, "tb-item-b", "disabled"), None);
    assert_eq!(
        attr(&tree, "tb-item-b", "aria-describedby").as_deref(),
        Some("tb-item-b-why-bubble")
    );
    let mut input = Input::new();
    input.focus_events(&mut tree, "tb-item-b", true);
    frame(&mut tree, 400.0, 120.0);
    assert_eq!(
        attr(&tree, "tb-item-b-why-bubble", "role").as_deref(),
        Some("tooltip")
    );
}

#[test]
fn a_command_state_becomes_the_reason() {
    let say = |k: &TextKey| format!("<{}>", k.as_str());
    assert_eq!(why_from_state(&State::Enabled, &say), None);
    assert_eq!(
        why_from_state(&State::Disabled { reason: "r".into() }, &say),
        Some(Why::Now("<r>".into()))
    );
    assert_eq!(
        why_from_state(&State::Unsupported { reason: "r".into() }, &say),
        Some(Why::Unsupported)
    );
}
