//! Bundle G5 (shell and HIG): the roles and names of the header bar, the card and the avatar group.
//!
//! Tests: attribute reads on the built elements (what an assistive technology reads).

mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use common::{attr, layout, serial};
use fs_components_widgets::{AvatarGroup, CardAction, FieldCard, Glyph, HeadBar, Person};

#[test]
fn the_header_bar_is_a_named_toolbar_with_a_heading() {
    let _g = serial();
    let bar = HeadBar::new()
        .id("hb")
        .left(div())
        .title("Contacts")
        .right(div())
        .build();
    let tree = layout(bar, 600.0, 80.0);
    assert_eq!(attr(&tree, "hb", "role").as_deref(), Some("toolbar"));
    assert_eq!(
        attr(&tree, "hb", "aria-orientation").as_deref(),
        Some("horizontal")
    );
    assert_eq!(attr(&tree, "hb", "aria-label").as_deref(), Some("Contacts"));
    assert_eq!(attr(&tree, "hb-title", "role").as_deref(), Some("heading"));
    assert_eq!(attr(&tree, "hb-title", "aria-level").as_deref(), Some("1"));
}

#[test]
fn a_header_bar_without_a_title_has_no_empty_name() {
    let _g = serial();
    let tree = layout(HeadBar::new().id("hb").left(div()).build(), 600.0, 80.0);
    assert_eq!(attr(&tree, "hb", "role").as_deref(), Some("toolbar"));
    assert_eq!(attr(&tree, "hb", "aria-label"), None);
}

#[test]
fn a_card_is_a_group_named_by_its_label_or_else_its_value() {
    let _g = serial();
    let act: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let with = FieldCard::new(Glyph::Mail, "Work", "ada@example.org").action(CardAction::new(
        Glyph::Copy,
        "Copy",
        Arc::clone(&act),
    ));
    let tree = layout(with.build("fc1"), 400.0, 120.0);
    assert_eq!(attr(&tree, "fc1", "role").as_deref(), Some("group"));
    assert_eq!(attr(&tree, "fc1", "aria-label").as_deref(), Some("Work"));
    let bare = FieldCard::new(Glyph::Mail, "", "ada@example.org");
    let tree = layout(bare.build("fc2"), 400.0, 120.0);
    assert_eq!(
        attr(&tree, "fc2", "aria-label").as_deref(),
        Some("ada@example.org")
    );
}

#[test]
fn an_avatar_group_names_its_people_and_counts_the_rest() {
    let _g = serial();
    let people: Vec<Person> = ["Ada Lovelace", "Bob Marley", "Cy Twombly", "Di Prince"]
        .iter()
        .enumerate()
        .map(|(i, n)| Person::new(format!("p{i}"), *n))
        .collect();
    let group = AvatarGroup::new(people).max(2).id("ag");
    assert_eq!(group.spoken(), "Ada Lovelace, Bob Marley, +2");
    let tree = layout(group.build(), 300.0, 80.0);
    assert_eq!(attr(&tree, "ag", "role").as_deref(), Some("group"));
    assert_eq!(
        attr(&tree, "ag", "aria-label").as_deref(),
        Some("Ada Lovelace, Bob Marley, +2")
    );
    assert_eq!(
        AvatarGroup::new(vec![Person::new("a", "Solo")]).spoken(),
        "Solo"
    );
}
