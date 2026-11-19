//! `checklist`: the group and its checkboxes (roles, `aria-checked`), the keys (arrows, Home, End, Space and Enter as the window's
//! click AND as raw keys, Delete and Backspace, Alt+Up and Alt+Down), the line for a new item (Enter), the remove button, the state
//! classes, the words in two languages, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, `frame` between presses as a window does.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::checklist::{Checklist, Item, Labels, STYLES, State, TEXTS};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<String>>>;

fn list(log: &Log) -> Checklist {
    let sink = Arc::clone(log);
    Checklist::new(
        "Packing",
        &Labels::default(),
        vec![
            Item::new("a", "Tent"),
            Item::new("b", "Stove").done(),
            Item::new("c", "Rope"),
        ],
    )
    .id("cl")
    .on_change(move |items| {
        sink.lock().unwrap().push(
            items
                .iter()
                .map(|i| format!("{}{}", i.key, if i.done { "+" } else { "" }))
                .collect::<Vec<_>>()
                .join(","),
        );
    })
}

fn show(c: &Checklist) -> RenderTree {
    layout_with(c.build(), 500.0, 300.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the item");
    tree.element_registry().has_class(node, class)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn keys(c: &Checklist) -> Vec<String> {
    c.items().iter().map(|i| i.key.clone()).collect()
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(
        (de.add.as_str(), de.remove.as_str()),
        ("Eintrag hinzufügen", "Entfernen")
    );
    assert_eq!(Labels::default().add, "Add an item");
}

#[test]
fn the_list_is_a_named_group_of_checkboxes_with_their_state() {
    let _g = serial();
    let tree = show(&list(&Log::default()));
    assert_eq!(attr(&tree, "cl", "role").as_deref(), Some("group"));
    assert_eq!(attr(&tree, "cl", "aria-label").as_deref(), Some("Packing"));
    assert_eq!(
        attr(&tree, "cl", "tabindex").as_deref(),
        Some("0"),
        "one tab stop"
    );
    assert_eq!(
        attr(&tree, "cl-item-a", "role").as_deref(),
        Some("checkbox")
    );
    assert_eq!(
        attr(&tree, "cl-item-a", "aria-checked").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "cl-item-b", "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "cl-item-a", "aria-label").as_deref(),
        Some("Tent")
    );
    assert_eq!(
        attr(&tree, "cl-remove-a", "aria-label").as_deref(),
        Some("Remove Tent")
    );
    assert!(has(&tree, "cl-item-b", "fs-check-item-done"));
    assert!(!has(&tree, "cl-item-a", "fs-check-item-done"));
    assert!(
        has(&tree, "cl-item-a", "fs-check-item-current"),
        "the cursor starts on the first"
    );
}

#[test]
fn arrows_home_end_move_the_cursor_without_wrap_and_the_keyboard_stays() {
    let _g = serial();
    let c = list(&Log::default());
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cl"));
    for (code, on) in [
        (40, "b"),
        (40, "c"),
        (40, "c"),
        (38, "b"),
        (36, "a"),
        (38, "a"),
        (35, "c"),
    ] {
        input.key(&mut tree, code);
        frame(&mut tree, 500.0, 300.0);
        assert!(
            has(&tree, &format!("cl-item-{on}"), "fs-check-item-current"),
            "key {code} -> {on}"
        );
        assert!(input.has_keyboard(&tree, "cl"));
    }
}

#[test]
fn space_and_enter_tick_once_as_the_windows_click_and_as_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let c = list(&log);
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cl"));
    input.key(&mut tree, 32);
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(
        attr(&tree, "cl-item-a", "aria-checked").as_deref(),
        Some("true")
    );
    input.key(&mut tree, 13);
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(
        attr(&tree, "cl-item-a", "aria-checked").as_deref(),
        Some("false")
    );
    input.key_event_to(&mut tree, "cl", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "cl", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "cl", KEY_DOWN, 65);
    assert!(
        !c.items()[0].done,
        "two raw ticks undo each other; another key does nothing"
    );
    assert_eq!(
        seen(&log),
        ["a+,b+,c", "a,b+,c", "a+,b+,c", "a,b+,c"],
        "each tick is heard once"
    );
}

#[test]
fn delete_and_backspace_remove_and_alt_arrows_put_an_item_in_another_place() {
    let _g = serial();
    let log = Log::default();
    let c = list(&log);
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cl"));
    input.press(&mut tree, 40, (false, false, true));
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(
        keys(&c),
        ["b", "a", "c"],
        "Alt+Down moved the first item down"
    );
    assert!(
        has(&tree, "cl-item-a", "fs-check-item-current"),
        "the cursor went with it"
    );
    input.press(&mut tree, 38, (false, false, true));
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(keys(&c), ["a", "b", "c"], "Alt+Up moved it back");
    input.press(&mut tree, 38, (false, false, true));
    assert_eq!(keys(&c), ["a", "b", "c"], "no move above the first");
    input.key(&mut tree, 35);
    input.press(&mut tree, 40, (false, false, true));
    assert_eq!(keys(&c), ["a", "b", "c"], "no move below the last");
    input.key(&mut tree, 46);
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(keys(&c), ["a", "b"], "Delete removed the last");
    assert!(
        has(&tree, "cl-item-b", "fs-check-item-current"),
        "the cursor went to the last item"
    );
    input.key_event_to(&mut tree, "cl", KEY_DOWN, 8);
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(keys(&c), ["a"], "a raw Backspace removed one more");
    assert_eq!(
        seen(&log).len(),
        4,
        "two moves and two removals are heard, the two refused moves are not"
    );
}

#[test]
fn an_item_before_the_cursor_that_goes_leaves_the_cursor_on_its_item() {
    let _g = serial();
    let c = list(&Log::default());
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cl"));
    input.key(&mut tree, 35);
    input.click(&mut tree, "cl-remove-a");
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(keys(&c), ["b", "c"]);
    assert!(
        has(&tree, "cl-item-c", "fs-check-item-current"),
        "the cursor stays on Rope"
    );
    assert!(!has(&tree, "cl-item-b", "fs-check-item-current"));
}

#[test]
fn a_click_ticks_and_moves_the_cursor_and_the_line_adds_an_item_on_enter() {
    let _g = serial();
    let log = Log::default();
    let c = list(&log);
    let mut tree = show(&c);
    let mut input = Input::new();
    input.click(&mut tree, "cl-item-c");
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(
        attr(&tree, "cl-item-c", "aria-checked").as_deref(),
        Some("true")
    );
    assert!(has(&tree, "cl-item-c", "fs-check-item-current"));
    input.click(&mut tree, "cl-add");
    for ch in "Map".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        frame(&mut tree, 500.0, 300.0);
    }
    input.key(&mut tree, 13);
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(c.items().last().map(|i| i.text.as_str()), Some("Map"));
    assert_eq!(c.items().len(), 4);
    assert!(
        tree.query_by_id("cl-item-n3").is_some(),
        "the new item has a key that was not used"
    );
    input.key(&mut tree, 13);
    assert_eq!(c.items().len(), 4, "an empty line makes no item");
}

#[test]
fn the_model_refuses_blanks_and_never_reuses_a_key() {
    let mut s = State::new(vec![Item::new("n1", "x")]);
    assert_eq!(s.add("  "), None);
    let k = s.add(" y ").expect("an item");
    assert_ne!(k, "n1");
    assert_eq!(s.items()[1].text, "y", "trimmed");
    assert!(s.remove(&k));
    let k2 = s.add("z").expect("an item");
    assert_ne!(k2, k, "a removed key is not given again");
    assert_eq!(s.toggle("nothing"), None);
    assert!(!s.remove("nothing"));
    let mut empty = State::new(Vec::new());
    for key in [Key::Down, Key::Space, Key::Delete, Key::Home] {
        assert_eq!(
            empty.press(KeyEvent::new(key)),
            fs_components_widgets::checklist::Outcome::Ignored
        );
    }
}
