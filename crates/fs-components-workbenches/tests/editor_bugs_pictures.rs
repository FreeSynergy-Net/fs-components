//! Findings b2, b3 and b4 of the hand test of fs-texts 0.0.13 (2026-11-16) on the writing surface of the Notion layout, as pictures
//! of the live widget after real events: b2 a new line at the end of the page makes the area grow too late, the top of the text
//! scrolls away (`view/wiki/surface_view.rs`: the area is built for the old number of lines when Blinc's own key handler
//! scrolls the cursor into view); b3 a double click marks no word (Blinc's text area has no word selection).
//!
//! Tests: pixel probes on `draw_to_png` of the widget (no window; skipped without graphics). Findings: fs-texts
//! `.scratch/wiki-0013-bilder/bugs/README.md`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // pixel arithmetic in a test

mod common;

use std::sync::Arc;

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use blinc_layout::renderer::ElementType;
use common::{Editable, Fake};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::{Labels, Layout, ReaderWidget, WikiLabels};

/// One test at a time may set Blinc up (process-wide state).
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ONE.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn editing() -> (ReaderWidget, RenderTree) {
    editing_page("# M\n\nalpha one\n\nbeta two\n")
}

fn editing_page(page: &str) -> (ReaderWidget, RenderTree) {
    let tmp = std::env::temp_dir().join(format!("fs-eb-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    let source = Editable::with(&[("m.md", page)]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(Layout::Notion, WikiLabels::default());
    widget.with_reader(|r| {
        r.open("m.md");
        r.begin_edit();
    });
    let tree = RenderTree::from_element(&widget.build(1000.0, 600.0));
    (widget, tree)
}

/// The picture of the widget as pixels, or `None` without graphics.
fn draw(widget: &ReaderWidget, name: &str) -> Option<image::RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-eb-{}", std::process::id())),
        std::path::PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let canvas = Canvas::new(1000, 600, fs_components_workbenches::styles()).with_window_styles();
    match draw_to_png(&canvas, |w, h| widget.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
                let _ = std::fs::remove_dir(&dir);
            }
            Some(picture)
        }
    }
}

/// The pixels of the writing area (below the title block) that `f` accepts.
fn count(img: &image::RgbaImage, f: fn([u8; 4]) -> bool) -> usize {
    img.enumerate_pixels()
        .filter(|(_, y, p)| *y >= 310 && f(p.0))
        .count()
}

/// The ink pixels from row `y` down (the writing area ends with the picture).
fn count_below(img: &image::RgbaImage, y: u32) -> usize {
    img.enumerate_pixels()
        .filter(|(_, row, p)| *row >= y && ink(p.0))
        .count()
}

/// The pixel over a white ground (the widget is drawn without the styles of a program: transparent where nothing is).
fn flat(c: [u8; 4]) -> [u8; 3] {
    let a = u16::from(c[3]);
    let over = |v: u8| ((u16::from(v) * a + 255 * (255 - a)) / 255) as u8;
    [over(c[0]), over(c[1]), over(c[2])]
}

fn ink(c: [u8; 4]) -> bool {
    flat(c).iter().all(|v| *v < 100)
}

/// Blue-ish and not dark: the marking (and the thin caret).
fn blue(c: [u8; 4]) -> bool {
    let c = flat(c);
    c[2] >= 240 && u16::from(c[0]) + 30 <= u16::from(c[2]) && c[0] < 235
}

#[test]
fn b2_enter_at_the_end_keeps_the_top_of_the_text_on_the_picture() {
    let _one = serial();
    let (widget, mut tree) = editing();
    widget.focus_block(1, 8);
    let Some(before) = draw(&widget, "b2-comp-vor-enter.png") else {
        return;
    };
    tree.broadcast_key_event(KEY_DOWN, 13, false, false, false, false);
    tree.process_pending_subtree_rebuilds();
    let after = draw(&widget, "b2-comp-vorher.png").unwrap();
    let (a, b) = (count(&before, ink), count(&after, ink));
    assert!(a > 100, "text on the first picture: {a}");
    // The first line ("alpha one", y 310 to 345) is on both pictures: it is what scrolled away.
    for (when, picture) in [("before", &before), ("after", &after)] {
        let first = count(picture, ink) - count_below(picture, 345);
        assert!(first > 50, "the first line is on the picture {when}: {first}");
    }
    assert!(
        b * 10 >= a * 9,
        "nothing of the text is lost: ink {a} before, {b} after"
    );
}

#[test]
fn b3_a_double_click_marks_the_word_and_draws_the_marking() {
    let _one = serial();
    let (widget, mut tree) = editing();
    widget.focus_block(0, 0);
    let Some(before) = draw(&widget, "b3-comp-vor-doppelklick.png") else {
        return;
    };
    // The text of the first block, as the laid-out tree has it: click 12 px into its first line.
    tree.compute_layout(1000.0, 600.0);
    let line = tree
        .iter_nodes()
        .find_map(|(n, node)| match &node.element_type {
            ElementType::Text(t) if t.content.starts_with("alpha") => Some(n),
            _ => None,
        })
        .expect("the line of the first block");
    let b = tree.get_absolute_bounds(line).expect("its place");
    let (x, y) = (b.x + 12.0, b.y + b.height / 2.0);
    let mut router = EventRouter::new();
    for _ in 0..2 {
        for events in [
            router.on_mouse_move(&tree, x, y),
            router.on_mouse_down(&tree, x, y, MouseButton::Left),
            router.on_mouse_up(&tree, x, y, MouseButton::Left),
        ] {
            for (n, e) in events {
                tree.dispatch_event(n, e, x, y);
            }
        }
    }
    tree.process_pending_subtree_rebuilds();
    let after = draw(&widget, "b3-comp-doppelklick-vorher.png").unwrap();
    assert_eq!(
        widget.selected_text().as_deref(),
        Some("alpha"),
        "the word is marked"
    );
    assert!(
        count(&after, blue) > count(&before, blue) + 150,
        "the marking is on the picture"
    );
}

/// The picture of `page` with a new block under "alpha one" that has a "/" in it: the block menu open (or closed again with Esc).
fn menu_picture(page: &str, name: &str, open: bool) -> Option<image::RgbaImage> {
    let (widget, mut tree) = editing_page(page);
    widget.focus_block(0, 9);
    tree.broadcast_key_event(KEY_DOWN, 13, false, false, false, false);
    tree.process_pending_subtree_rebuilds();
    widget.type_in_block(1, 0, "/");
    tree.process_pending_subtree_rebuilds();
    assert!(widget.block_menu_open(), "the slash opens the menu");
    if !open {
        tree.broadcast_key_event(KEY_DOWN, 27, false, false, false, false);
        tree.process_pending_subtree_rebuilds();
        assert!(!widget.block_menu_open(), "Esc closes it");
    }
    draw(&widget, name)
}

/// The pixels that differ between two pictures: their frame (left, top, right, bottom) and their number inside `within`.
fn differing(
    a: &image::RgbaImage,
    b: &image::RgbaImage,
    within: Option<[u32; 4]>,
) -> (Option<[u32; 4]>, usize) {
    let mut frame: Option<[u32; 4]> = None;
    let mut inside = 0;
    for (x, y, p) in a.enumerate_pixels() {
        if p == b.get_pixel(x, y) {
            continue;
        }
        frame = Some(frame.map_or([x, y, x, y], |f| [f[0].min(x), f[1].min(y), f[2].max(x), f[3].max(y)]));
        inside += usize::from(within.is_some_and(|w| x >= w[0] && x <= w[2] && y >= w[1] && y <= w[3]));
    }
    (frame, inside)
}

/// b4: "second paragraph" shone through the block menu (the text under the menu was drawn over its ground). The menu opens
/// over the block below the cursor; the same menu over a page without that block must look the same inside, pixel for pixel
/// (the inside: the menu without its frame and its shadow, found as what the open menu changes on the page).
#[test]
fn b4_the_block_menu_hides_the_text_under_it() {
    let _one = serial();
    let page = "# M\n\nalpha one\n\nbeta two\n";
    let Some(over_text) = menu_picture(page, "b4-menue-ueber-text.png", true) else {
        return;
    };
    let alone = menu_picture("# M\n\nalpha one\n", "b4-menue-allein.png", true).unwrap();
    let closed = menu_picture("# M\n\nalpha one\n", "b4-ohne-menue.png", false).unwrap();
    let text = menu_picture(page, "b4-text-ohne-menue.png", false).unwrap();
    let (frame, _) = differing(&alone, &closed, None);
    let f = frame.expect("the open menu changes the picture");
    let inner = [f[0] + 24, f[1] + 24, f[2] - 24, f[3] - 24];
    let (_, under) = differing(&text, &closed, Some(inner));
    assert!(under > 50, "the text lies under the menu: {under} pixels");
    let (_, shining) = differing(&over_text, &alone, Some(inner));
    assert_eq!(shining, 0, "pixels of the menu that show the text under it");
}
