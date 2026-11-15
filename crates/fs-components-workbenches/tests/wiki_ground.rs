//! The wiki layouts follow the theme (finding of the acceptance pictures: Notion and Confluence painted fixed whites and greys,
//! so under Craft the dark text stood on white), and a code block has a ground that stands off the page of the reader, in the
//! default style and in Craft (finding U8).
//!
//! Tests: pixels of the real pictures (skipped without graphics or fonts). A process has one style, so the Craft checks are
//! in `wiki_ground_craft.rs`.

mod common;

use common::pages::{page_ground, tallest_ground_run, wiki_picture};
use fs_components_workbenches::Layout;

#[test]
fn a_code_block_stands_off_the_page_in_the_default_style() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture(None, layout, (1000, 1100), "ground-default.png") else {
            return;
        };
        let run = tallest_ground_run(&img, 520, page_ground(&img));
        assert!(
            run >= 20,
            "{layout:?}: the code block has a ground of its own ({run} px)"
        );
    }
}
