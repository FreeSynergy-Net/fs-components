//! `popover`: trigger state (`aria-haspopup`, `aria-expanded`, `aria-controls`), panel role and name, Escape and the close mark,
//! the keyboard going back to the trigger, the words in two languages.
//!
//! Tests: attribute reads on the built host, an in-process pointer and keyboard, the request to the host for the keyboard.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::{Input, attr, frame, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::popover::{Labels, Popover, TEXTS};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<String>>>;

fn popover(log: &Log) -> Popover {
    let (focus, closed) = (Arc::clone(log), Arc::clone(log));
    Popover::new(&Labels::default(), "Filters")
        .id("pv")
        .focus_to(move |id| focus.lock().unwrap().push(format!("focus {id}")))
        .on_close(move || closed.lock().unwrap().push("closed".to_owned()))
}

fn host(pv: &Popover) -> Div {
    pv.build(
        || div().attr("tabindex", "0").w(80.0).h(30.0),
        || {
            div()
                .child(text("Only mine"))
                .child(div().id("pv-inside").attr("tabindex", "0").w(40.0).h(20.0))
        },
    )
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.close, "Schließen");
    assert_eq!(Labels::default().close, "Close");
}

#[test]
fn the_trigger_tells_its_state_and_the_panel_is_a_named_dialog() {
    let _g = serial();
    let pv = popover(&Log::default());
    let mut tree = layout(host(&pv), 400.0, 300.0);
    assert_eq!(
        attr(&tree, "pv-trigger", "aria-haspopup").as_deref(),
        Some("dialog")
    );
    assert_eq!(
        attr(&tree, "pv-trigger", "aria-expanded").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "pv-trigger", "aria-controls").as_deref(),
        Some("pv-panel")
    );
    assert!(tree.query_by_id("pv-panel").is_none(), "closed: no panel");
    let mut input = Input::new();
    input.click(&mut tree, "pv-trigger");
    frame(&mut tree, 400.0, 300.0);
    assert!(pv.is_open());
    assert_eq!(
        attr(&tree, "pv-trigger", "aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(attr(&tree, "pv-panel", "role").as_deref(), Some("dialog"));
    assert_eq!(
        attr(&tree, "pv-panel", "aria-label").as_deref(),
        Some("Filters")
    );
    assert!(
        tree.query_by_id("pv-inside").is_some(),
        "the content is there"
    );
}

#[test]
fn a_second_click_on_the_trigger_closes_it() {
    let _g = serial();
    let pv = popover(&Log::default());
    let mut tree = layout(host(&pv), 400.0, 300.0);
    let mut input = Input::new();
    input.click(&mut tree, "pv-trigger");
    frame(&mut tree, 400.0, 300.0);
    input.click(&mut tree, "pv-trigger");
    frame(&mut tree, 400.0, 300.0);
    assert!(!pv.is_open());
    assert!(tree.query_by_id("pv-panel").is_none());
}

#[test]
fn escape_closes_it_and_the_keyboard_goes_back_to_the_trigger() {
    let _g = serial();
    let log = Log::default();
    let pv = popover(&log);
    let mut tree = layout(host(&pv), 400.0, 300.0);
    let mut input = Input::new();
    pv.open();
    frame(&mut tree, 400.0, 300.0);
    input.key(&mut tree, 27);
    frame(&mut tree, 400.0, 300.0);
    assert!(!pv.is_open());
    assert_eq!(seen(&log), ["focus pv-trigger", "closed"]);
    assert!(!pv.close(), "already closed: nothing more happens");
    assert_eq!(seen(&log).len(), 2);
}

#[test]
fn the_close_mark_is_a_named_tab_stop_that_closes_by_key_or_click() {
    let _g = serial();
    let log = Log::default();
    let pv = popover(&log);
    let mut tree = layout(host(&pv), 400.0, 300.0);
    let mut input = Input::new();
    pv.open();
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(attr(&tree, "pv-close", "role").as_deref(), Some("button"));
    assert_eq!(
        attr(&tree, "pv-close", "aria-label").as_deref(),
        Some("Close")
    );
    assert!(input.tab_to(&mut tree, "pv-close"));
    input.key(&mut tree, 13);
    assert!(!pv.is_open(), "Enter on the mark closes");
    pv.open();
    frame(&mut tree, 400.0, 300.0);
    input.click(&mut tree, "pv-close");
    assert!(!pv.is_open(), "a click on the mark closes");
    assert_eq!(
        seen(&log)
            .iter()
            .filter(|e| *e == "focus pv-trigger")
            .count(),
        2
    );
}

#[test]
fn enter_as_a_raw_key_on_the_close_mark_closes_it_too() {
    // Mutant 17 of the first run (`Key::Enter` and `Key::Activate` dropped from the key arm) survived because the click the window
    // makes of Enter closed it anyway. A raw key event sent to the focused mark (a frontend that delivers the keys itself) does not
    // go through a click, so only the key arm can close it.
    use blinc_core::events::event_types::KEY_DOWN;
    let _g = serial();
    let pv = popover(&Log::default());
    let mut tree = layout(host(&pv), 400.0, 300.0);
    let mut input = Input::new();
    pv.open();
    frame(&mut tree, 400.0, 300.0);
    assert!(input.tab_to(&mut tree, "pv-close"));
    input.key_event_to(&mut tree, "pv-close", KEY_DOWN, 13);
    assert!(!pv.is_open(), "Enter as a raw key closes it");
}
