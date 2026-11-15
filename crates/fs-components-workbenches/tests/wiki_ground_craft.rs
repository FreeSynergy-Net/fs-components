//! Under Craft the Notion and Confluence layouts are dark like the style (no fixed white), and a code block still stands off the
//! page. One style per process, hence a file of its own (see `wiki_ground.rs`).

mod common;

use common::pages::{page_ground, tallest_ground_run, wiki_picture};
use fs_components_workbenches::Layout;

#[test]
fn the_layouts_follow_craft_and_the_code_block_stands_off() {
    for layout in [Layout::Notion, Layout::Confluence] {
        let Some(img) = wiki_picture(Some("craft"), layout, (1000, 1100), "ground-craft.png")
        else {
            return;
        };
        let ground = page_ground(&img);
        // Craft's window is dark in the dark scheme and light grey (F2F1EE) in the light one: never the fixed #ffffff.
        assert_ne!(
            &ground[..3],
            &[255, 255, 255],
            "{layout:?}: the page follows the style ({ground:?})"
        );
        // The side column too: its pixel is not a fixed white or the fixed greys of the old sheet.
        let side = img.get_pixel(5, 900).0;
        for fixed in [[247, 246, 243], [247, 248, 249], [255, 255, 255]] {
            assert_ne!(&side[..3], &fixed, "{layout:?}: the side follows the style");
        }
        let run = tallest_ground_run(&img, 520, ground);
        assert!(
            run >= 20,
            "{layout:?}: the code block has a ground of its own ({run} px)"
        );
    }
}
