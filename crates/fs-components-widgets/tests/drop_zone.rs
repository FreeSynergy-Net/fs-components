//! `drop-zone`: the keyboard way (Enter, Space, click pick once), the over state, the words in two languages, the role and name.
//!
//! Tests: attribute reads, an in-process keyboard and pointer, and the file-drag events the window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::{FILE_DRAG_LEAVE, FILE_DRAG_OVER, FILE_DROP};
use common::{Input, attr, frame, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::drop_zone::{DropZone, Labels, TEXTS};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<&'static str>>>;

fn zone(log: &Log) -> DropZone {
    let (pick, drop) = (Arc::clone(log), Arc::clone(log));
    DropZone::new(&Labels::default())
        .id("dz")
        .title("Photos")
        .on_pick(move || pick.lock().unwrap().push("pick"))
        .on_drop(move || drop.lock().unwrap().push("drop"))
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert!(de.hint.contains("Eingabetaste"));
    assert!(Labels::default().hint.contains("Enter"));
}

#[test]
fn it_is_a_button_named_by_its_title_and_its_two_ways() {
    let _g = serial();
    let z = zone(&Log::default());
    let tree = layout(z.build(), 400.0, 200.0);
    assert_eq!(attr(&tree, "dz", "role").as_deref(), Some("button"));
    assert_eq!(attr(&tree, "dz", "tabindex").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "dz", "aria-label").as_deref(),
        Some("Photos. Drop files here, or press Enter to choose files")
    );
}

#[test]
fn enter_picks_once_and_a_click_picks_once() {
    let _g = serial();
    let log = Log::default();
    let mut tree = layout(zone(&log).build(), 400.0, 200.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "dz"));
    input.key(&mut tree, 13);
    assert_eq!(*log.lock().unwrap(), ["pick"], "Enter picks exactly once");
    input.click(&mut tree, "dz");
    assert_eq!(log.lock().unwrap().len(), 2, "a click picks once more");
}

#[test]
fn files_over_it_change_the_state_and_the_words_and_a_drop_ends_it() {
    let _g = serial();
    let log = Log::default();
    let z = zone(&log);
    let mut tree = layout(z.build(), 400.0, 200.0);
    let node = tree.query_by_id("dz").unwrap();
    tree.dispatch_event(node, FILE_DRAG_OVER, 1.0, 1.0);
    frame(&mut tree, 400.0, 200.0);
    assert!(z.is_over());
    tree.dispatch_event(node, FILE_DRAG_LEAVE, 1.0, 1.0);
    assert!(!z.is_over());
    z.drag_over(true);
    let node = tree.query_by_id("dz").unwrap();
    tree.dispatch_event(node, FILE_DROP, 1.0, 1.0);
    assert!(!z.is_over(), "a drop ends the hover");
    assert_eq!(*log.lock().unwrap(), ["drop"]);
}

#[test]
fn files_over_it_draw_the_zone_again_with_the_over_state() {
    // Mutant g5-15 of the run (the change check of `drag_over` turned round) survived: the state was stored but nothing was drawn again.
    let _g = serial();
    let z = zone(&Log::default());
    let mut tree = layout(z.build(), 400.0, 200.0);
    let node = tree.query_by_id("dz").unwrap();
    assert!(!tree.element_registry().has_class(node, "fs-drop-zone-over"));
    tree.dispatch_event(node, FILE_DRAG_OVER, 1.0, 1.0);
    frame(&mut tree, 400.0, 200.0);
    let node = tree.query_by_id("dz").unwrap();
    assert!(
        tree.element_registry().has_class(node, "fs-drop-zone-over"),
        "the zone must be drawn again with the over state"
    );
}
