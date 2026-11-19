//! `chip`: the row as a named list, chips as list items, remove buttons named in words (de/en), Left/Right/Home/End, Delete and
//! Backspace (window keys AND raw) remove the chip under the cursor, the cursor class, the pointer, the callback, add.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, `frame` between presses.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::chip::{Chip, ChipList, Labels, STYLES, TEXTS};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<String>>>;

fn chips(log: &Log) -> ChipList {
    let l = Arc::clone(log);
    ChipList::new(
        "Keywords",
        &Labels::default(),
        vec![
            Chip::new("a", "alpha"),
            Chip::new("b", "beta"),
            Chip::new("c", "gamma"),
        ],
    )
    .id("ch")
    .on_remove(move |k| l.lock().unwrap().push(k.to_owned()))
}

fn show(c: &ChipList) -> RenderTree {
    layout_with(c.build(), 500.0, 120.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the chip");
    tree.element_registry().has_class(node, class)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    assert_eq!(
        Labels::for_locale(&Locale::parse("de").unwrap()).remove,
        "Entfernen"
    );
    assert_eq!(Labels::default().remove, "Remove");
}

#[test]
fn the_row_is_a_named_list_of_items_with_named_remove_buttons() {
    let _g = serial();
    let tree = show(&chips(&Log::default()));
    assert_eq!(attr(&tree, "ch", "role").as_deref(), Some("list"));
    assert_eq!(attr(&tree, "ch", "aria-label").as_deref(), Some("Keywords"));
    assert_eq!(attr(&tree, "ch", "tabindex").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "ch-chip-b", "role").as_deref(),
        Some("listitem")
    );
    assert_eq!(
        attr(&tree, "ch-remove-b", "role").as_deref(),
        Some("button")
    );
    assert_eq!(
        attr(&tree, "ch-remove-b", "aria-label").as_deref(),
        Some("Remove beta")
    );
    assert!(
        has(&tree, "ch-chip-a", "fs-chip-current"),
        "the cursor starts on the first"
    );
    assert!(!has(&tree, "ch-chip-b", "fs-chip-current"));
}

#[test]
fn arrows_home_end_move_the_cursor_without_wrap_and_with_a_frame_between() {
    let _g = serial();
    let c = chips(&Log::default());
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "ch"));
    for (code, on) in [
        (39, "b"),
        (39, "c"),
        (39, "c"),
        (37, "b"),
        (36, "a"),
        (37, "a"),
        (35, "c"),
    ] {
        input.key(&mut tree, code);
        frame(&mut tree, 500.0, 120.0);
        assert!(
            has(&tree, &format!("ch-chip-{on}"), "fs-chip-current"),
            "key {code} -> {on}"
        );
        assert!(input.has_keyboard(&tree, "ch"));
    }
    assert!(!has(&tree, "ch-chip-a", "fs-chip-current"));
}

#[test]
fn delete_and_backspace_remove_the_chip_under_the_cursor_as_window_keys_and_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let c = chips(&log);
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "ch"));
    input.key(&mut tree, 39);
    input.key(&mut tree, 46);
    frame(&mut tree, 500.0, 120.0);
    assert_eq!(c.keys(), ["a", "c"], "Delete removed beta");
    assert!(tree.query_by_id("ch-chip-b").is_none());
    assert!(
        has(&tree, "ch-chip-c", "fs-chip-current"),
        "the cursor kept its place"
    );
    input.key_event_to(&mut tree, "ch", KEY_DOWN, 8);
    frame(&mut tree, 500.0, 120.0);
    assert_eq!(c.keys(), ["a"], "a raw Backspace removed gamma");
    assert!(
        has(&tree, "ch-chip-a", "fs-chip-current"),
        "the cursor went to the last chip"
    );
    input.key_event_to(&mut tree, "ch", KEY_DOWN, 46);
    input.key(&mut tree, 46);
    assert_eq!(c.keys(), Vec::<String>::new());
    assert_eq!(
        seen(&log),
        ["b", "c", "a"],
        "each removal is heard once; an empty row says nothing"
    );
    input.key_event_to(&mut tree, "ch", KEY_DOWN, 65);
    assert_eq!(seen(&log).len(), 3);
}

#[test]
fn the_remove_button_removes_once_and_add_refuses_a_key_that_is_there() {
    let _g = serial();
    let log = Log::default();
    let c = chips(&log);
    let mut tree = show(&c);
    let mut input = Input::new();
    input.click(&mut tree, "ch-remove-a");
    frame(&mut tree, 500.0, 120.0);
    assert_eq!(c.keys(), ["b", "c"]);
    assert_eq!(seen(&log), ["a"]);
    assert!(!c.add(Chip::new("b", "again")));
    assert!(c.add(Chip::new("d", "delta")));
    frame(&mut tree, 500.0, 120.0);
    assert_eq!(
        attr(&tree, "ch-remove-d", "aria-label").as_deref(),
        Some("Remove delta")
    );
    assert_eq!(c.keys(), ["b", "c", "d"]);
}

#[test]
fn removing_a_chip_before_the_cursor_keeps_the_cursor_on_its_chip() {
    let _g = serial();
    let c = chips(&Log::default());
    assert!(c.add(Chip::new("d", "delta")));
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "ch"));
    input.key(&mut tree, 39);
    input.key(&mut tree, 39);
    frame(&mut tree, 500.0, 120.0);
    assert!(has(&tree, "ch-chip-c", "fs-chip-current"));
    input.click(&mut tree, "ch-remove-a");
    frame(&mut tree, 500.0, 120.0);
    assert!(
        has(&tree, "ch-chip-c", "fs-chip-current"),
        "the cursor stays on gamma"
    );
    assert!(!has(&tree, "ch-chip-b", "fs-chip-current"));
}
