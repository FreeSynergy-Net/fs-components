//! Findings U7 and U8 of the user test of 2026-11-14: the text of a table is as big as the text of a paragraph (it was smaller),
//! and a code block has a ground and a frame and no more room around its lines than it needs.
//!
//! Tests: the real element drawn with the default look of Blinc's widgets (skipped without fonts); heights are read from the
//! layout, ink from the picture.
//!
//! The picture `u8-codeblock.png` is written only when the environment variable `FS_COMP_SHOTS` names a directory (see
//! `common::draw_frames`); without it nothing is kept. The page here is the theme's background, which the reader's page paints too (it
//! was a fixed white before); the reader's own pictures, in every style, are checked by `fs-components-workbenches/tests/wiki_code_ground.rs`.

mod common;

use common::{draw_tree, has_fonts, layout_with, serial};
use fs_components_widgets::TextView;
use fs_components_widgets::text::{Align, Block, Inline};

fn ua() -> &'static str {
    blinc_layout::ua_styles::UA_STYLES
}

/// Rows with dark ink in `x0..x1`, from `y0` on: (first, last).
fn ink_rows(img: &image::RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> Option<(u32, u32)> {
    let rows: Vec<u32> = (y0..y1.min(img.height()))
        .filter(|&y| (x0..x1.min(img.width())).any(|x| img.get_pixel(x, y)[0] < 110))
        .collect();
    Some((*rows.first()?, *rows.last()?))
}

#[test]
fn the_text_of_a_table_is_as_big_as_the_text_of_a_paragraph() {
    let _g = serial();
    if !has_fonts() {
        eprintln!("skipped: no fonts");
        return;
    }
    let word = "Handlungshoheit";
    let blocks = vec![
        Block::Paragraph(Inline::plain(word)),
        Block::Table {
            align: vec![Align::None],
            rows: vec![vec![Inline::plain("head")], vec![Inline::plain(word)]],
        },
    ];
    let tree = layout_with(TextView::new(blocks).reading().build(), 500.0, 200.0, ua());
    let img = draw_tree(&tree, 500, 200, "u7-tabellenschrift.png");
    // The paragraph is the first line of ink; the table cell with the same word is the last.
    let (p0, p1) = ink_rows(&img, 0, 400, 0, 40).expect("paragraph");
    let (c0, c1) = ink_rows(&img, 0, 400, 95, 200).expect("cell");
    let (paragraph, cell) = (p1 - p0, c1 - c0);
    assert!(
        cell + 1 >= paragraph,
        "the cell text is as high as the paragraph text ({cell} px against {paragraph} px)"
    );
}

#[test]
fn a_code_block_has_a_ground_and_a_frame_and_little_room_around_its_lines() {
    let _g = serial();
    if !has_fonts() {
        eprintln!("skipped: no fonts");
        return;
    }
    let blocks = vec![Block::Code {
        lang: String::new(),
        text: "cool".into(),
    }];
    let tree = layout_with(TextView::new(blocks).reading().build(), 400.0, 200.0, ua());
    let img = draw_tree(&tree, 400, 200, "u8-codeblock.png");
    // The block: the frame is a line of a different grey than the page along the top and the sides.
    let page = *img.get_pixel(390, 190);
    let differs = |x: u32, y: u32| (0..3).any(|c| img.get_pixel(x, y)[c].abs_diff(page[c]) >= 4);
    let col: Vec<u32> = (0..200).filter(|&y| differs(100, y)).collect();
    let (top, bottom) = (
        *col.first().expect("a frame or ground"),
        *col.last().unwrap(),
    );
    // A ground: the middle of the block is clearly not the colour of the page (a frame alone is not enough). The reader paints
    // the theme's background (`--background`) as its page, which is what this harness draws too; a bare 4 of 255 is too faint.
    let mid = *img.get_pixel(200, u32::midpoint(top, bottom));
    assert!(
        (0..3).any(|c| mid[c].abs_diff(page[c]) >= 8),
        "the block has a ground that stands off the page ({mid:?} on {page:?})"
    );
    // One line of code: the box is not much higher than the line and its padding (it was 81 px for one line).
    assert!(
        bottom - top <= 52,
        "one line takes {} px of box ({}..{})",
        bottom - top,
        top,
        bottom
    );
}
