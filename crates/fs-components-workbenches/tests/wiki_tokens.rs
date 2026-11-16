//! The wiki layouts in the default style: the edge of an empty box and the arrow and symbols are seen (3:1, WCAG 2.1 1.4.11), the
//! code ground has its own token, the narrow window shows whole rows, and the small points of the 0.0.20 review.
//!
//! Tests: pixels of the real pictures (skipped without graphics or fonts, an error while drawing fails). A process has one
//! style, so the Craft checks are in `wiki_ground_craft.rs`; the measures are in `common/wiki_checks.rs`.

mod common;

use common::pages::{contrast, wiki_picture, wiki_picture_steps};
use common::wiki_checks as check;
use fs_components_workbenches::{Layout, Step};

#[test]
fn an_empty_box_has_an_edge_that_is_seen() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture_steps(
            None,
            layout,
            (1000, 1100),
            vec![Step::Open("boxes.md".into())],
            "box.png",
        ) else {
            return;
        };
        check::box_edge_is_seen(&img, layout);
    }
}

#[test]
fn the_measure_of_the_box_edge_can_fail() {
    // Counter-test: the edge of 0.0.20 (205,205,205 on white, 1.59:1) is below the measure.
    let c = contrast(
        image::Rgba([205, 205, 205, 255]),
        image::Rgba([255, 255, 255, 255]),
    );
    assert!(c < check::SEEN, "{c}");
}

#[test]
fn the_arrow_and_the_symbols_are_seen() {
    let steps = || {
        vec![
            Step::Open("handbuch.md".into()),
            Step::Expand("handbuch.md".into()),
        ]
    };
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture_steps(None, layout, (1000, 1100), steps(), "arrow.png") else {
            return;
        };
        check::chevron_is_seen(&img, layout);
        if layout == Layout::Notion {
            check::head_icons_are_seen(&img);
        }
    }
}

#[test]
fn the_code_block_has_a_ground_of_its_own_that_a_hover_does_not_swallow() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture(None, layout, (1000, 1100), "code.png") else {
            return;
        };
        check::code_block_stands_off(&img, layout, 1.05);
    }
    // The hover ground of a row and of a block is `--accent-subtle`: the code ground must be a token of its own.
    let rule = fs_components_widgets::text_view::STYLES
        .lines()
        .find(|l| l.starts_with(".fs-tv-code {"))
        .expect("the rule of the code block");
    assert!(
        rule.contains("background: var(--code-bg"),
        "the ground is the token: {rule}"
    );
}

#[test]
fn a_narrow_window_shows_whole_rows_of_the_tree() {
    for layout in [Layout::Notion, Layout::Confluence] {
        check::stacked_tree_has_whole_rows(layout, None);
    }
}

#[test]
fn the_search_field_of_the_confluence_bar_stays_in_the_window() {
    check::search_fits_in_the_window(None);
}

#[test]
fn the_title_keeps_its_weight_while_editing_and_starts_where_the_text_starts() {
    check::title_keeps_weight_and_alignment(None);
}
