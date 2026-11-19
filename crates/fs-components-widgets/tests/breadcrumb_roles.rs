//! `breadcrumb` (G1): a named navigation, links, the place as `aria-current`, the words in two languages, Enter and Space as the
//! window's click AND as raw keys (once each), the pointer once.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use common::{Input, attr, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb, Labels, TEXTS};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<String>>>;

fn crumbs(n: usize) -> Vec<Crumb> {
    (0..n).map(|i| Crumb::new(format!("k{i}"), format!("Step {i}"))).collect()
}

fn way(log: &Log, n: usize) -> Breadcrumb {
    let l = Arc::clone(log);
    Breadcrumb::new(crumbs(n), "bc").on_choose(Arc::new(move |k| l.lock().unwrap().push(k.to_owned())))
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.name, "Pfad");
    assert_eq!(de.earlier, "Frühere Schritte");
    assert_eq!(Labels::default().name, "Breadcrumb");
}

#[test]
fn the_row_is_a_named_navigation_the_steps_are_links_and_the_last_is_the_place() {
    let _g = serial();
    let b = way(&Log::default(), 3).labels(&Labels::for_locale(&Locale::parse("de").unwrap()));
    let tree = layout(b.build(600.0), 600.0, 60.0);
    assert_eq!(attr(&tree, "bc-crumbs", "role").as_deref(), Some("navigation"));
    assert_eq!(attr(&tree, "bc-crumbs", "aria-label").as_deref(), Some("Pfad"));
    assert_eq!(attr(&tree, "bc-crumb-0", "role").as_deref(), Some("link"));
    assert_eq!(attr(&tree, "bc-crumb-1", "role").as_deref(), Some("link"));
    assert_eq!(attr(&tree, "bc-crumb-0", "aria-current"), None);
    assert_eq!(attr(&tree, "bc-crumb-2", "aria-current").as_deref(), Some("page"));
    assert_eq!(attr(&tree, "bc-crumb-2", "role"), None, "the place is not a link");
    assert_eq!(attr(&tree, "bc-gap-1", "aria-hidden").as_deref(), Some("true"), "a separator is not read aloud");
    assert_eq!(attr(&tree, "bc-gap-2", "aria-hidden").as_deref(), Some("true"));
}

#[test]
fn the_left_out_steps_have_a_name_in_words() {
    let _g = serial();
    let tree = layout(way(&Log::default(), 12).build(300.0), 300.0, 60.0);
    assert_eq!(attr(&tree, "bc-crumb-hidden", "role").as_deref(), Some("link"));
    assert_eq!(attr(&tree, "bc-crumb-hidden", "aria-label").as_deref(), Some("Earlier steps"));
}

#[test]
fn enter_and_space_press_a_step_once_as_the_windows_click_and_as_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let mut tree = layout(way(&log, 3).build(600.0), 600.0, 60.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "bc-crumb-0"));
    input.key(&mut tree, 13);
    assert_eq!(*log.lock().unwrap(), ["k0"], "Enter as the window's click: once");
    input.key(&mut tree, 32);
    assert_eq!(log.lock().unwrap().len(), 2, "Space as the window's click: once");
    input.key_event_to(&mut tree, "bc-crumb-0", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "bc-crumb-0", KEY_DOWN, 13);
    assert_eq!(log.lock().unwrap().len(), 4, "a raw Space and a raw Enter");
    input.key_event_to(&mut tree, "bc-crumb-0", KEY_DOWN, 65);
    assert_eq!(log.lock().unwrap().len(), 4, "another key does nothing");
}

#[test]
fn the_pointer_presses_a_step_once() {
    let _g = serial();
    let log = Log::default();
    let mut tree = layout(way(&log, 3).build(600.0), 600.0, 60.0);
    let mut input = Input::new();
    input.click(&mut tree, "bc-crumb-1");
    assert_eq!(*log.lock().unwrap(), ["k1"]);
}
