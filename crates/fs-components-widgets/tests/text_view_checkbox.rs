//! The checkbox of a table cell is the real checkbox component, the same one as in a list, and it knows the form of the user test
//! (finding E1/E2 of 2026-11-14: `- [ ] Test?` stayed text, and the boxes were drawn as emoji characters).
//!
//! Tests: the real element, laid out with the default look of Blinc's own widgets; an in-process mouse clicks the boxes.

// Pixel arithmetic on small picture sizes: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

mod common;

use common::serial;
use fs_components_widgets::TextView;
use fs_components_widgets::text::{Block, Inline, ListItem};

fn row(cells: &[&str]) -> Vec<Inline> {
    cells.iter().map(|c| Inline::plain(*c)).collect()
}

/// The cell of the user test: `- [ ] Test?` (a list marker typed in the cell) is a checkbox, like `[ ]` alone.
#[test]
fn the_cell_of_the_user_test_is_a_real_checkbox_the_same_as_in_a_list() {
    use common::{Input, layout_with};
    use std::sync::{Arc, Mutex};
    let _g = serial();
    let clicks: Arc<Mutex<Vec<(usize, bool)>>> = Arc::default();
    let sink = Arc::clone(&clicks);
    let blocks = vec![
        Block::Table {
            align: vec![],
            rows: vec![
                row(&["Name", "Test"]),
                row(&["Kal", "- [ ] Test?"]),
                row(&["Ada", "[x]"]),
            ],
        },
        Block::List {
            ordered: false,
            start: 1,
            items: vec![ListItem::new(vec![Block::Paragraph(Inline::plain("one"))]).task(false)],
        },
    ];
    assert_eq!(
        fs_components_widgets::count_tasks(&blocks),
        3,
        "two cells and one list item"
    );
    let view =
        TextView::new(blocks).on_task(Arc::new(move |i, d| sink.lock().unwrap().push((i, d))), 0);
    let mut tree = layout_with(
        view.build(),
        600.0,
        300.0,
        blinc_layout::ua_styles::UA_STYLES,
    );
    let mut input = Input::new();
    for id in ["fs-tv-task-0", "fs-tv-task-1", "fs-tv-task-2"] {
        assert!(
            tree.query_by_id(id).is_some(),
            "{id}: the same box component in a cell and in a list"
        );
        input.click(&mut tree, id);
    }
    assert_eq!(*clicks.lock().unwrap(), [(0, true), (1, false), (2, true)]);
    // No glyph imitation of a box (the characters U+2610 and U+2611) in any of the places that draw one.
    for (name, src) in [
        ("table.rs", include_str!("../src/text_view/table.rs")),
        ("list.rs", include_str!("../src/text_view/list.rs")),
        ("task.rs", include_str!("../src/text_view/task.rs")),
    ] {
        assert!(
            !src.contains("2610") && !src.contains("2611"),
            "{name} draws a box as a character"
        );
    }
}

/// Finding U2 (2026-11-14): the box was small and sat above the baseline of its line of text. Now it is as tall as the text and its
/// middle is the middle of the line of ink.
#[test]
fn the_box_sits_on_the_line_of_its_text_and_is_as_tall_as_the_text() {
    use common::{draw_tree, has_fonts, layout_with};
    let _g = serial();
    if !has_fonts() {
        eprintln!("skipped: no fonts");
        return;
    }
    let blocks = vec![Block::List {
        ordered: false,
        start: 1,
        items: vec![
            ListItem::new(vec![Block::Paragraph(Inline::plain(
                "Offen und sehr gut sichtbar",
            ))])
            .task(false),
            ListItem::new(vec![Block::Paragraph(Inline::plain("Erledigt"))]).task(true),
        ],
    }];
    let view = TextView::new(blocks)
        .reading()
        .on_task(std::sync::Arc::new(|_, _| {}), 0);
    let tree = layout_with(view.build(), 400.0, 120.0, blink_ua());
    let img = draw_tree(&tree, 400, 120, "u2-kaestchen-grundlinie.png");
    for id in ["fs-tv-task-0", "fs-tv-task-1"] {
        let node = tree.query_by_id(id).expect(id);
        let b = tree.get_absolute_bounds(node).expect("bounds");
        assert!(
            b.height >= 17.0,
            "{id}: the box is as tall as body text ({} px)",
            b.height
        );
        // The ink of the label right of the box, in the rows of the box (a little more: the text may be taller).
        let (x0, y0, y1) = (
            (b.x + b.width + 6.0) as u32,
            (b.y - 6.0).max(0.0) as u32,
            (b.y + b.height + 6.0) as u32,
        );
        let rows: Vec<u32> = (y0..y1)
            .filter(|&y| (x0..x0 + 160).any(|x| img.get_pixel(x, y)[0] < 110))
            .collect();
        let (top, bottom) = (*rows.first().expect("ink"), *rows.last().unwrap());
        let ink_middle = f32::midpoint(top as f32, bottom as f32);
        let box_middle = b.y + b.height / 2.0;
        assert!(
            (ink_middle - box_middle).abs() <= 3.0,
            "{id}: the box (middle {box_middle}) is on the line of its text (ink middle {ink_middle})"
        );
    }
}

fn blink_ua() -> &'static str {
    blinc_layout::ua_styles::UA_STYLES
}
