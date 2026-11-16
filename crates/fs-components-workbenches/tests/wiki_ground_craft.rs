//! Under Craft the Notion and Confluence layouts are dark like the style: the page and the side column are the tokens of the style
//! (not a fixed colour), a code block stands off the page by 1.3:1, an empty box, the arrow of the selected row and the symbols are
//! seen (3:1, WCAG 2.1 1.4.11). One style per process, hence a file of its own (see `wiki_tokens.rs`).
//!
//! Tests: pixels of the real pictures; the measures are in `common/wiki_checks.rs`.

mod common;

use blinc_theme::{ColorToken, ThemeState};
use common::pages::{contrast, page_ground, wiki_picture, wiki_picture_steps};
use common::wiki_checks as check;
use fs_components_workbenches::{Layout, Step};

/// The pixel of a token of the active theme (opaque, as it is drawn on nothing).
fn token(t: ColorToken) -> [u8; 3] {
    let c = ThemeState::get().color(t);
    // A colour channel is 0.0..=1.0 here; rounding to a byte is what is meant.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(c.r), byte(c.g), byte(c.b)]
}

#[test]
fn the_layouts_are_the_tokens_of_craft() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture(Some("craft"), layout, (1000, 1100), "ground-craft.png")
        else {
            return;
        };
        // The page is the surface token and the side column the background token of the style: not "something other than white",
        // the value itself (a new fixed colour would fail here).
        let ground = page_ground(&img);
        assert_eq!(
            ground[..3],
            token(ColorToken::Surface),
            "{layout:?}: the page is the surface token"
        );
        let side = img.get_pixel(5, 900).0;
        assert_eq!(
            side[..3],
            token(ColorToken::Background),
            "{layout:?}: the side is the background token"
        );
        // The text of the page reads on it (4.5:1).
        let text = image::Rgba([
            token(ColorToken::TextPrimary)[0],
            token(ColorToken::TextPrimary)[1],
            token(ColorToken::TextPrimary)[2],
            255,
        ]);
        assert!(contrast(text, image::Rgba(ground)) >= 4.5);
        check::code_block_stands_off(&img, layout, 1.3);
    }
}

#[test]
fn an_empty_box_has_an_edge_that_is_seen_in_craft() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let steps = vec![Step::Open("boxes.md".into())];
        let Some(img) =
            wiki_picture_steps(Some("craft"), layout, (1000, 1100), steps, "box-craft.png")
        else {
            return;
        };
        check::box_edge_is_seen(&img, layout);
    }
}

#[test]
fn the_arrow_and_the_symbols_are_seen_in_craft() {
    let steps = || {
        vec![
            Step::Open("handbuch.md".into()),
            Step::Expand("handbuch.md".into()),
        ]
    };
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture_steps(
            Some("craft"),
            layout,
            (1000, 1100),
            steps(),
            "arrow-craft.png",
        ) else {
            return;
        };
        check::chevron_is_seen(&img, layout);
        if layout == Layout::Notion {
            check::head_icons_are_seen(&img);
        }
    }
}

#[test]
fn a_narrow_window_shows_whole_rows_of_the_tree_in_craft() {
    for layout in [Layout::Notion, Layout::Confluence] {
        check::stacked_tree_has_whole_rows(layout, Some("craft"));
    }
}

#[test]
fn the_search_field_stays_in_the_window_in_craft() {
    check::search_fits_in_the_window(Some("craft"));
}

#[test]
fn the_title_keeps_its_weight_while_editing_in_craft() {
    check::title_keeps_weight_and_alignment(Some("craft"));
}
