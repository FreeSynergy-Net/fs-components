//! Pictures of the bundle G5 (drop zone, header bar, card, avatar group) in a FreeSynergy style, light and dark, with the theme guard
//! of `styled.rs` ([THEME-WAECHTER]); kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene in the style Admin, light and dark, and asserts that something was drawn and that the dark picture
//! differs from the light one; the pictures are for a person to look at (rule: pictures before release).

mod common;
#[path = "common/styled.rs"]
mod styled;

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::drop_zone::{DropZone, Labels};
use fs_components_widgets::head_bar::{Press, head_button};
use fs_components_widgets::{AvatarGroup, CardAction, FieldCard, Glyph, HeadBar, Person};
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 5] = [
    ("dropzone", 600, 220, dropzone),
    ("dropzone-over", 600, 220, dropzone_over),
    ("headbar", 600, 160, headbar),
    ("card", 600, 180, card),
    ("avatars", 400, 180, avatars),
];

fn dropzone() -> Div {
    let z = DropZone::new(&Labels::default())
        .id("g5-dz1")
        .title("Photos");
    div().w(460.0).child(z.build())
}

fn dropzone_over() -> Div {
    let z = DropZone::new(&Labels::default())
        .id("g5-dz2")
        .title("Photos");
    z.drag_over(true);
    div().w(460.0).child(z.build())
}

fn headbar() -> Div {
    let press: Press = Arc::new(|| {});
    div().w(480.0).child(
        HeadBar::new()
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
            .build(),
    )
}

fn card() -> Div {
    let act: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let card = FieldCard::new(Glyph::Mail, "Work", "ada@example.org").action(CardAction::new(
        Glyph::Copy,
        "Copy",
        act,
    ));
    div().w(480.0).child(card.build("g5-card"))
}

fn avatars() -> Div {
    let people: Vec<Person> = ["Ada Lovelace", "Bob Marley", "Cy Twombly", "Di Prince"]
        .iter()
        .enumerate()
        .map(|(i, n)| Person::new(format!("p{i}"), *n))
        .collect();
    div().child(AvatarGroup::new(people).max(3).id("g5-ag").build())
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("g5-{name}.png");
        let light = draw_styled(build, w, h, StyleId::Admin, false, &css, &file);
        let dark = draw_styled(build, w, h, StyleId::Admin, true, &css, &file);
        for (img, mode) in [(&light, "light"), (&dark, "dark")] {
            let ground = ground_of(img);
            let ink = img
                .enumerate_pixels()
                .filter(|(x, y, _)| *x >= STRIP + 4 || *y >= SWATCH + 4)
                .filter(|(_, _, p)| {
                    (0..3)
                        .map(|i| i32::from(p.0[i].abs_diff(ground[i])))
                        .sum::<i32>()
                        > 40
                })
                .count();
            assert!(ink > 300, "{name} {mode}: nearly empty ({ink} ink pixels)");
        }
        assert_ne!(
            ground_of(&light),
            ground_of(&dark),
            "{name}: dark is not dark"
        );
    }
}
