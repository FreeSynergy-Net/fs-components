//! The colour roles of a style (kinds of file, shade, paper, tiles of a plan) and the style in use.
//!
//! Tests: model tests over the tables; no drawing.

use fs_components_styles::styles::colour_roles::{
    IconKind, export_ink, icon_paper, icon_shade, kind_colour, on_colour, plan_tile, sheet,
};
use fs_components_styles::{Rgba, StyleId, current_style, set_current_style};

#[test]
fn admin_is_the_base_table_the_widgets_had_before() {
    let want = [
        (IconKind::Folder, 0x00F1_B531),
        (IconKind::Image, 0x002F_9E6E),
        (IconKind::Text, 0x006B_7A8C),
        (IconKind::Archive, 0x00A0_6B2C),
        (IconKind::Audio, 0x00B0_5BB5),
        (IconKind::Video, 0x00C0_504D),
        (IconKind::Pdf, 0x00D0_4437),
        (IconKind::Sheet, 0x002E_8B57),
        (IconKind::Doc, 0x003B_6FC4),
        (IconKind::Slides, 0x00D9_7A2B),
        (IconKind::Code, 0x004A_86C8),
        (IconKind::Unknown, 0x008A_949E),
        (IconKind::Desktop, 0x003B_82C4),
        (IconKind::Downloads, 0x002F_9E6E),
        (IconKind::Documents, 0x00C4_8A2A),
        (IconKind::Pictures, 0x0045_8F8A),
        (IconKind::Music, 0x00A0_5BB5),
        (IconKind::Videos, 0x00C0_504D),
    ];
    assert_eq!(want.len(), IconKind::ALL.len(), "every kind has its value");
    for (kind, rgb) in want {
        assert_eq!(
            kind_colour(StyleId::Admin, kind),
            Rgba::hex(rgb),
            "{kind:?}"
        );
    }
    assert_eq!(plan_tile(StyleId::Admin, 0), Rgba::hex(0x002F_7BF5));
    assert_eq!(on_colour(StyleId::Admin), Rgba::hex(0x00FF_FFFF));
    assert_eq!(export_ink(StyleId::Admin), Rgba::hex(0x0044_4444));
    assert_eq!(icon_paper(StyleId::Admin), Rgba::hex(0x00FA_FCFF));
    assert_eq!((icon_shade(false).a, icon_shade(true).a), (56, 61));
}

#[test]
fn a_change_of_style_changes_the_colours() {
    for kind in IconKind::ALL {
        let admin = kind_colour(StyleId::Admin, kind);
        let apple = kind_colour(StyleId::Apple, kind);
        assert_ne!(admin, apple, "{kind:?} must follow the style");
        // Apple is lighter, Simple darker, Craft lighter still: the direction is the style's, not by chance
        let sum = |c: Rgba| u32::from(c.r) + u32::from(c.g) + u32::from(c.b);
        assert!(sum(apple) > sum(admin), "{kind:?}: Apple lightens");
        assert!(
            sum(kind_colour(StyleId::Simple, kind)) < sum(admin),
            "{kind:?}: Simple darkens"
        );
        assert!(
            sum(kind_colour(StyleId::Craft, kind)) > sum(apple),
            "{kind:?}: Craft lightens most"
        );
    }
    assert_ne!(plan_tile(StyleId::Admin, 3), plan_tile(StyleId::Apple, 3));
}

#[test]
fn the_sheet_gives_seven_tiles_and_the_colour_on_a_colour_for_every_style() {
    for id in StyleId::ALL {
        let css = sheet(id);
        for n in 0..7 {
            assert!(
                css.contains(&format!("--plan-tile-{n}: #")),
                "{} tile {n}",
                id.name()
            );
        }
        assert!(css.contains("--on-colour: #ffffff;"), "{}", id.name());
        assert!(css.starts_with(":root {") && css.trim_end().ends_with('}'));
    }
    // the tile after the last one is the first again (a plan with more people than tiles)
    assert_eq!(plan_tile(StyleId::Admin, 7), plan_tile(StyleId::Admin, 0));
}

#[test]
fn the_style_in_use_is_admin_until_one_is_chosen_and_can_change() {
    // one test: the value is process wide
    assert_eq!(current_style(), StyleId::Admin);
    for id in StyleId::ALL {
        set_current_style(id);
        assert_eq!(current_style(), id);
    }
    set_current_style(StyleId::Admin);
    assert_eq!(current_style(), StyleId::Admin);
}
