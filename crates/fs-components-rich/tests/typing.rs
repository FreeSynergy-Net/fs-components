//! Typing on the writing surface through the handlers of the built element: shortcuts, Enter, Backspace, the arrow keys over
//! blocks, the click on a task box. Each case has a counter-case that must not change the page.
//!
//! Tests: event tests, the keys and the mouse as the window gives them (see `common`).

mod common;

use common::{BACKSPACE, DOWN, END, HOME, LEFT, RIGHT, Typist, UP};
use fs_components_editor::Doc;

fn empty(style: &str) -> Typist {
    let mut t = Typist::new(style, &Doc::new(vec![String::new()]), 800.0, 700.0);
    t.click(10.0, 10.0);
    assert!(
        t.surface.focused(),
        "a click gives the surface the keyboard"
    );
    t
}

#[test]
fn typed_shortcuts_make_the_blocks_and_the_page_comes_out_as_markdown() {
    let mut t = empty("standard");
    t.type_text("## Plan\nThings to do:\n[] write\nagenda\n\n1. one\ntwo\n\n> said\n\n```rust\nfn a() {}\n\n---\nend");
    assert_eq!(
        t.markdown(),
        "## Plan\n\nThings to do:\n\n- [ ] write\n- [ ] agenda\n\n1. one\n2. two\n\n> said\n\n```rust\nfn a() {}\n```\n\n---\n\nend\n"
    );
}

#[test]
fn counter_case_without_the_space_nothing_turns() {
    let mut t = empty("standard");
    t.type_text("##Plan\n[]x\n-y");
    assert_eq!(t.markdown(), "##Plan\n\n[]x\n\n-y\n");
}

#[test]
fn keys_without_the_keyboard_do_nothing() {
    let mut t = Typist::new("standard", &Doc::parse("abc\n"), 800.0, 400.0);
    t.type_text("zzz");
    t.key(BACKSPACE);
    assert_eq!(
        t.markdown(),
        "abc\n",
        "the surface has not been clicked: it has no keyboard"
    );
}

#[test]
fn the_cursor_walks_over_the_blocks_with_the_arrow_keys() {
    let mut t = Typist::new(
        "standard",
        &Doc::parse("# Head\n\n- [ ] a\n- [ ] b\n\ntail\n"),
        800.0,
        500.0,
    );
    t.click(400.0, 5.0);
    let at = |t: &Typist| (t.surface.cursor().path, t.surface.cursor().at);
    assert_eq!(
        at(&t).0,
        vec![0],
        "the click on the heading puts the cursor in it"
    );
    t.key(END);
    assert_eq!(at(&t), (vec![0], 4));
    t.key(RIGHT);
    assert_eq!(
        at(&t),
        (vec![1, 0, 0], 0),
        "Right at the end goes to the next block"
    );
    t.key(DOWN);
    assert_eq!(at(&t), (vec![1, 1, 0], 0));
    t.key(DOWN);
    assert_eq!(at(&t), (vec![2], 0));
    t.key(UP);
    t.key(UP);
    t.key(UP);
    assert_eq!(
        at(&t),
        (vec![0], 0),
        "Up goes back to the heading and stops there"
    );
    t.key(LEFT);
    assert_eq!(at(&t), (vec![0], 0), "nothing left of the start");
    t.key(HOME);
    assert_eq!(
        t.markdown(),
        "# Head\n\n- [ ] a\n- [ ] b\n\ntail\n",
        "walking changes nothing"
    );
}

#[test]
fn a_click_on_the_box_ticks_it_and_a_second_click_unticks_it() {
    let mut t = Typist::new("standard", &Doc::parse("- [ ] a\n- [x] b\n"), 800.0, 300.0);
    t.click_box(&[0, 0, 0]);
    assert_eq!(t.markdown(), "- [x] a\n- [x] b\n");
    t.click_box(&[0, 1, 0]);
    assert_eq!(t.markdown(), "- [x] a\n- [ ] b\n");
    t.click_box(&[0, 0, 0]);
    assert_eq!(t.markdown(), "- [ ] a\n- [ ] b\n");
    assert!(t.surface.focused());
    // Counter-case: a click on the text of an item ticks nothing, it puts the cursor there.
    let g = t.surface.geometry();
    let leaf = g.leaf(&[0, 1, 0]).expect("leaf");
    t.click(leaf.x + 1.0, leaf.y + 2.0);
    assert_eq!(t.markdown(), "- [ ] a\n- [ ] b\n");
    assert_eq!(t.surface.cursor().path, vec![0, 1, 0]);
}

#[test]
fn a_click_puts_the_cursor_between_the_characters_nearest_to_it() {
    let mut t = Typist::new("standard", &Doc::parse("abcdef\n"), 800.0, 300.0);
    let g = t.surface.geometry();
    let (leaf, from) = (
        g.leaf(&[0]).expect("leaf").clone(),
        g.caret(&[0], 3).expect("caret"),
    );
    t.click(from.x + 1.0, leaf.y + 3.0);
    assert_eq!(t.surface.cursor().at, 3);
    t.click(0.5, leaf.y + 3.0);
    assert_eq!(
        t.surface.cursor().at,
        0,
        "at the left edge of the text: the start"
    );
    t.click(700.0, leaf.y + 3.0);
    assert_eq!(t.surface.cursor().at, 6, "right of the text: the end");
}

#[test]
fn backspace_and_enter_go_through_the_handlers_too() {
    let mut t = empty("standard");
    t.type_text("- one\ntwo");
    t.key(BACKSPACE);
    t.key(BACKSPACE);
    t.key(BACKSPACE);
    assert_eq!(t.markdown(), "- one\n- \n");
    t.key(BACKSPACE);
    assert_eq!(
        t.markdown(),
        "- one\n\n\n",
        "the empty item leaves the list as a paragraph"
    );
}
