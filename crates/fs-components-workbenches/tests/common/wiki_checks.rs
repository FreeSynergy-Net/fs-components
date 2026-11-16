//! What the pictures of the wiki layouts must show, in every style: the same measures for the default style and for Craft (a
//! process has one style, so each style has a test file of its own that calls these).
//!
//! Tests: test helpers (pure functions over pictures).

use super::pages::{
    contrast, empty_box_edge, ink_extent, page_ground, selected_row, strong_in, wiki_picture,
    wiki_picture_steps,
};
use fs_components_workbenches::{Layout, Step};

/// WCAG 2.1, 1.4.11: what a part of a control that must be seen needs against its ground.
pub const SEEN: f32 = 3.0;

/// The edge of an empty box (a task, a table cell) has 3:1 against the page.
pub fn box_edge_is_seen(img: &image::RgbaImage, layout: Layout) {
    let ground = page_ground(img);
    let edge = empty_box_edge(
        img,
        ground,
        if layout == Layout::Notion { 290 } else { 185 },
    )
    .expect("an empty box is on the page");
    let c = contrast(image::Rgba(edge), image::Rgba(ground));
    assert!(
        c >= SEEN,
        "{layout:?}: the edge {edge:?} of an empty box on {ground:?} has {c:.2}:1, wanted {SEEN}:1"
    );
}

/// The arrow of the selected, opened row of the tree is drawn with 3:1 against the selection (it is muted text colour on the
/// selection ground otherwise, which vanished in Craft).
pub fn chevron_is_seen(img: &image::RgbaImage, layout: Layout) {
    let plain = image::Rgba(img.get_pixel(250, 1000).0);
    let (top, bottom) = selected_row(img, 250, 70, plain.0).expect("a selected row");
    let mid = u32::midpoint(top, bottom);
    let ground = *img.get_pixel(250, mid);
    let n = strong_in(img, 4..24, mid - 8..=mid + 8, ground, SEEN);
    assert!(
        n >= 6,
        "{layout:?}: the arrow of the selected row has {n} pixels with {SEEN}:1 against {ground:?}"
    );
}

/// The history and the star in the top line of the Notion layout have 3:1 against the line.
pub fn head_icons_are_seen(img: &image::RgbaImage) {
    let ground = *img.get_pixel(600, 3);
    for (name, xs) in [("history", 858..882), ("star", 924..948)] {
        let n = strong_in(img, xs, 10..=34, ground, SEEN);
        assert!(
            n >= 12,
            "{name}: {n} pixels with {SEEN}:1 against {ground:?}"
        );
    }
}

/// The code block stands off the page by `least` (contrast of its ground against the page ground).
pub fn code_block_stands_off(img: &image::RgbaImage, layout: Layout, least: f32) {
    let ground = page_ground(img);
    let off = |y: u32| (0..3).any(|c| img.get_pixel(520, y)[c].abs_diff(ground[c]) >= 3);
    let (mut best, mut run, mut at) = ((0, 0), 0, 0);
    for y in 0..img.height() {
        if off(y) {
            if run == 0 {
                at = y;
            }
            run += 1;
            if run > best.0 {
                best = (run, at);
            }
        } else {
            run = 0;
        }
    }
    assert!(
        best.0 >= 20,
        "{layout:?}: a code block has a ground of its own"
    );
    // a few rows inside the block (its frame is a thin line at the edge)
    let inside = *img.get_pixel(520, best.1 + 8);
    let c = contrast(inside, image::Rgba(ground));
    assert!(
        c >= least,
        "{layout:?}: the code ground {inside:?} on {ground:?} has {c:.2}:1, wanted {least}:1"
    );
}

/// A window of 380 x 400 px stacks the column above the page, and the tree in it is whole rows: its height is a multiple of the
/// row, the last row is not cut by the line under the column.
pub fn stacked_tree_has_whole_rows(layout: Layout, style: Option<&str>) {
    let steps = vec![
        Step::Open("handbuch.md".into()),
        Step::Expand("handbuch.md".into()),
    ];
    let Some(img) = wiki_picture_steps(style, layout, (380, 400), steps, "stacked.png") else {
        return;
    };
    // the line under the column: the first row from 100 px down in which three columns have one colour that their rows above and
    // below do not have
    let line = |y: u32| {
        let at = |x: u32, yy: u32| img.get_pixel(x, yy).0;
        [2, 190, 377]
            .iter()
            .all(|&x| at(x, y) == at(2, y) && at(x, y) != at(x, y - 1) && at(x, y) != at(x, y + 1))
    };
    let end = (100..img.height() - 1)
        .find(|&y| line(y))
        .expect("the line under the column");
    // up from the line: the free rows, the row of the page that is open (it has the selection as its ground), above it the head
    let plain = img.get_pixel(370, end - 2).0;
    let mut top = end - 2;
    while img.get_pixel(370, top).0 == plain {
        top -= 1;
    }
    let selection = img.get_pixel(370, top).0;
    while img.get_pixel(370, top).0 == selection {
        top -= 1;
    }
    let top = top + 1;
    let row = if layout == Layout::Notion { 28 } else { 30 };
    let room = end - top;
    // Whole rows: the room is a number of rows (the rows start a pixel or two below the head), here the three pages of the
    // picture; a third of 400 px leaves less than a row after the head, so the old height showed one row and a half.
    let n = (room + 3) / row;
    assert!(
        room.abs_diff(n * row) <= 3 && n == 3,
        "{layout:?}: the tree is {room} px high, not three whole rows of {row} px"
    );
}

/// The search field of the Confluence bar stays in the window at 560 px.
pub fn search_fits_in_the_window(style: Option<&str>) {
    let Some(img) = wiki_picture(style, Layout::Confluence, (560, 400), "narrow-bar.png") else {
        return;
    };
    let ground = img.get_pixel(5, 3).0;
    let (_, last) = ink_extent(&img, 300..560, 0..50, ground).expect("the field is drawn");
    assert!(last + 8 <= 560, "the search field ends at {last} of 560 px");
}

/// The title of the page is as wide while editing as while reading (the same weight) and the way, the title and the text start at
/// the same x (the block's own 6 px are taken into the title).
pub fn title_keeps_weight_and_alignment(style: Option<&str>) {
    let Some(read) = wiki_picture(style, Layout::Notion, (1000, 800), "title-read.png") else {
        return;
    };
    let edit = wiki_picture_steps(
        style,
        Layout::Notion,
        (1000, 800),
        vec![Step::Open("handbuch.md".into()), Step::Edit],
        "title-edit.png",
    )
    .expect("fonts work: the first picture was drawn");
    let ground = page_ground(&read);
    let span =
        |img: &image::RgbaImage| ink_extent(img, 280..700, 190..262, ground).expect("a title");
    let (a, b) = (span(&read), span(&edit));
    let (wr, we) = (a.1 - a.0, b.1 - b.0);
    assert!(
        wr.abs_diff(we) <= 4,
        "the title is {wr} px wide reading and {we} px editing: another weight"
    );
    assert!(
        a.0.abs_diff(b.0) <= 3,
        "the title starts at {} reading and {} editing",
        a.0,
        b.0
    );
    let crumbs = ink_extent(&read, 270..700, 12..32, ground)
        .expect("the way to the page")
        .0;
    let text = ink_extent(&read, 270..700, 305..330, ground)
        .expect("a paragraph")
        .0;
    assert!(
        crumbs.abs_diff(text) <= 2,
        "the way starts at {crumbs}, the text at {text}"
    );
    assert!(
        a.0.abs_diff(text) <= 3,
        "the title starts at {}, the text at {text}",
        a.0
    );
}
