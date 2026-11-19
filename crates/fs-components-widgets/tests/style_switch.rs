//! A change of style while the program runs: the icons change with it (the colour of a kind of file is a token of the style), the
//! picture pair admin -> apple shows it (kept when `FS_COMP_SHOTS` names a directory, with the theme guard of `styled.rs`).
//!
//! Tests: `fs_components_render::set_style` between two pictures of the same view; the view is built again after the change (an icon
//! takes its colour when it is built); the pixels of the icons differ, the folder's colour is the style's.

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::{Div, div};
use common::serial;
use fs_components_render::{set_style, use_style};
use fs_components_styles::StyleId;
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::{PinnedKind, pinned_icon};
use styled::{STRIP, SWATCH, draw_styled};

fn view() -> Div {
    let mut row = div().flex_row().gap(12.0);
    for glyph in [
        Glyph::Folder,
        Glyph::FileImage,
        Glyph::FilePdf,
        Glyph::FileSheet,
        Glyph::FileCode,
    ] {
        row = row.child(icon(glyph, 56.0, Tone::Kind));
    }
    row.child(pinned_icon(PinnedKind::Pictures, 56.0))
        .child(pinned_icon(PinnedKind::Music, 56.0))
}

/// How many pixels of `first` and `second` (outside the strip of the guard) differ clearly.
fn differing(first: &image::RgbaImage, second: &image::RgbaImage) -> usize {
    let mut count = 0;
    for (col, row, left) in first.enumerate_pixels() {
        if col < STRIP + 4 && row < SWATCH + 4 {
            continue;
        }
        let right = second.get_pixel(col, row);
        let gap: i32 = (0..3)
            .map(|i| (i32::from(left.0[i]) - i32::from(right.0[i])).abs())
            .sum();
        count += usize::from(gap > 48);
    }
    count
}

#[test]
fn the_icons_change_with_the_style_of_a_running_window() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    let (w, h) = (780, 190);
    set_style(StyleId::Admin, false, &css);
    let before = draw_styled(
        view,
        w,
        h,
        StyleId::Admin,
        false,
        &css,
        "switch-1-admin.png",
    );
    set_style(StyleId::Apple, false, &css);
    let after = draw_styled(
        view,
        w,
        h,
        StyleId::Apple,
        false,
        &css,
        "switch-2-apple.png",
    );
    let changed = differing(&before, &after);
    assert!(
        changed > 1_000,
        "the picture hardly changed ({changed} pixels)"
    );
    // back again: the same pixels as at first (nothing stays behind from the other style)
    set_style(StyleId::Admin, false, &css);
    let again = draw_styled(
        view,
        w,
        h,
        StyleId::Admin,
        false,
        &css,
        "switch-3-admin-again.png",
    );
    assert_eq!(
        differing(&before, &again),
        0,
        "back to Admin is the first picture"
    );
    use_style(None);
}

#[test]
fn the_folder_takes_the_colour_of_the_style_in_use() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for id in [StyleId::Admin, StyleId::Apple, StyleId::Craft] {
        set_style(id, false, &css);
        let want = fs_components_styles::styles::colour_roles::kind_colour(
            id,
            fs_components_styles::styles::colour_roles::IconKind::Folder,
        );
        assert_eq!(
            fs_components_widgets::icon::kind_colour(Glyph::Folder),
            Some(want),
            "{}",
            id.name()
        );
    }
    use_style(None);
}

#[test]
fn an_outline_icon_takes_the_icon_set_of_the_style_in_use() {
    // Mutant S-f of the run (the icon set of Admin always) survived: the view of the first test had no outline icon, only kinds.
    let _g = serial();
    let css = fs_components_widgets::styles();
    let outline = || {
        div()
            .flex_row()
            .gap(12.0)
            .child(icon(Glyph::Close, 96.0, Tone::Normal))
            .child(icon(Glyph::Search, 96.0, Tone::Normal))
    };
    set_style(StyleId::Admin, false, &css);
    let admin = draw_styled(
        outline,
        340,
        220,
        StyleId::Admin,
        false,
        &css,
        "outline-admin.png",
    );
    set_style(StyleId::Apple, false, &css);
    let apple = draw_styled(
        outline,
        340,
        220,
        StyleId::Apple,
        false,
        &css,
        "outline-apple.png",
    );
    let changed = differing(&admin, &apple);
    assert!(
        changed > 100,
        "the line of the icons did not change ({changed} pixels)"
    );
    use_style(None);
}
