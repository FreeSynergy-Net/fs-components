//! Pictures of the bundle G5 (drop zone, header bar, card, avatar group), kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws each block offscreen with the theme and the default look of Blinc's buttons and asserts only that something was
//! drawn; the pictures are for a person to look at (rule: pictures before release).

mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use common::{background, draw_window, ink, serial};
use fs_components_widgets::drop_zone::{DropZone, Labels};
use fs_components_widgets::head_bar::{Press, head_button};
use fs_components_widgets::{AvatarGroup, CardAction, FieldCard, Glyph, HeadBar, Person};

fn drawn(img: &image::RgbaImage) {
    let (w, h) = img.dimensions();
    assert!(
        ink(img, (0, 0, w, h), background(img)) > 200,
        "nothing drawn"
    );
}

#[test]
fn drop_zone_at_rest_and_with_files_over_it() {
    let _g = serial();
    let rest = DropZone::new(&Labels::default())
        .id("g5-dz1")
        .title("Photos");
    drawn(&draw_window(rest.build(), 460, 160, "g5-dropzone.png"));
    let over = DropZone::new(&Labels::default())
        .id("g5-dz2")
        .title("Photos");
    over.drag_over(true);
    drawn(&draw_window(over.build(), 460, 160, "g5-dropzone-over.png"));
}

#[test]
fn header_bar_card_and_avatar_group() {
    let _g = serial();
    let press: Press = Arc::new(|| {});
    let bar = HeadBar::new()
        .id("g5-hb")
        .left(head_button(
            "g5-hb-back",
            Glyph::Close,
            "Close",
            false,
            &press,
        ))
        .title("Contacts")
        .right(div())
        .build();
    drawn(&draw_window(bar, 480, 70, "g5-headbar.png"));
    let act: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let card = FieldCard::new(Glyph::Mail, "Work", "ada@example.org").action(CardAction::new(
        Glyph::Copy,
        "Copy",
        act,
    ));
    drawn(&draw_window(card.build("g5-card"), 480, 90, "g5-card.png"));
    let people: Vec<Person> = ["Ada Lovelace", "Bob Marley", "Cy Twombly", "Di Prince"]
        .iter()
        .enumerate()
        .map(|(i, n)| Person::new(format!("p{i}"), *n))
        .collect();
    let group = AvatarGroup::new(people).max(3).id("g5-ag");
    drawn(&draw_window(group.build(), 300, 90, "g5-avatars.png"));
}
