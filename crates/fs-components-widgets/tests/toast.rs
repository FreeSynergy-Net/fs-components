//! The toast stack: kinds, time (injected, no waiting), hold, the stack limit, the action, the keys and the pointer.
//!
//! Tests: Model tests over the stack with explicit milliseconds, an in-process pointer and keyboard over the built stack.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, frame, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::states::Severity;
use fs_components_widgets::toast::{Labels, MAX_VISIBLE, TEXTS, Toast, Toasts};
use fs_components_words::{Locale, Localized};

fn stack() -> Toasts {
    Toasts::new(&Labels::default())
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.close, "Meldung schließen");
}

#[test]
fn info_ok_and_warning_go_by_themselves_an_error_stays() {
    let _g = serial();
    let t = stack();
    for kind in Severity::ALL {
        t.push_at(Toast::new(kind, "x"), 1_000);
    }
    assert_eq!(t.count(), 4);
    assert_eq!(t.tick(1_000 + 3_999), 0, "nothing has run out yet");
    assert_eq!(t.tick(1_000 + 4_000), 1, "ok goes after 4 s");
    assert_eq!(t.tick(1_000 + 5_000), 1, "info after 5 s");
    assert_eq!(t.tick(1_000 + 8_000), 1, "a warning after 8 s");
    assert_eq!(t.tick(1_000 + 3_600_000), 0, "an error stays for an hour");
    assert_eq!(t.count(), 1);
}

#[test]
fn a_toast_can_ask_for_its_own_time_or_to_stay() {
    let _g = serial();
    let t = stack();
    let short = t.push_at(Toast::new(Severity::Info, "a").lasts(100), 0);
    let stays = t.push_at(Toast::new(Severity::Ok, "b").stays(), 0);
    t.tick(100);
    assert!(!t.dismiss(short), "the short one is gone");
    t.tick(1_000_000);
    assert!(t.dismiss(stays), "the other one is still there");
}

#[test]
fn a_held_toast_keeps_its_time_and_goes_after_the_rest() {
    let _g = serial();
    let t = stack();
    let id = t.push_at(Toast::new(Severity::Info, "a"), 0); // goes at 5000
    t.hold_at(id, true, 3_000); // 2000 left
    assert_eq!(t.tick(100_000), 0, "held: it stays however long");
    t.hold_at(id, false, 100_000);
    assert_eq!(t.tick(101_999), 0);
    assert_eq!(t.tick(102_000), 1, "released: the 2 s that were left run");
}

#[test]
fn older_toasts_wait_behind_a_count_and_come_back() {
    let _g = serial();
    let t = stack();
    for i in 0..MAX_VISIBLE + 2 {
        t.push_at(Toast::new(Severity::Error, &format!("e{i}")), 0);
    }
    let mut tree = layout(t.build(), 420.0, 600.0);
    assert!(
        tree.query_by_id("toasts-more").is_some(),
        "the count of waiting ones"
    );
    // the newest are in view: ids 3..=6 of 1..=6
    assert!(tree.query_by_id("toasts-1").is_none() && tree.query_by_id("toasts-2").is_none());
    assert!(tree.query_by_id("toasts-3").is_some() && tree.query_by_id("toasts-6").is_some());
    assert!(t.dismiss(6));
    frame(&mut tree, 420.0, 600.0);
    assert!(
        tree.query_by_id("toasts-2").is_some(),
        "one waiting toast comes into view"
    );
}

#[test]
fn the_action_runs_and_closes_the_toast() {
    let _g = serial();
    let log: Arc<Mutex<Vec<&str>>> = Arc::default();
    let seen = Arc::clone(&log);
    let t = stack();
    let id = t.push_at(
        Toast::new(Severity::Ok, "Removed")
            .action("Undo", move || seen.lock().unwrap().push("undo")),
        0,
    );
    let mut tree = layout(t.build(), 420.0, 200.0);
    let mut input = Input::new();
    input.click(&mut tree, &format!("toasts-{id}-action"));
    assert_eq!(*log.lock().unwrap(), ["undo"]);
    assert_eq!(t.count(), 0);
    assert!(!t.run_action(id), "gone: nothing to run");
    assert!(!t.run_action(t.push_at(Toast::new(Severity::Info, "no action"), 0)));
}

#[test]
fn the_close_mark_is_a_tab_stop_and_escape_or_enter_or_a_click_close() {
    let _g = serial();
    let t = stack();
    let a = t.push_at(Toast::new(Severity::Error, "one"), 0);
    let b = t.push_at(Toast::new(Severity::Error, "two"), 0);
    let c = t.push_at(Toast::new(Severity::Warn, "three"), 0);
    let mut tree = layout(t.build(), 420.0, 400.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, &format!("toasts-{a}-close")));
    input.key(&mut tree, 27);
    assert_eq!(t.count(), 2, "Escape closed the focused one");
    frame(&mut tree, 420.0, 400.0);
    assert!(input.tab_to(&mut tree, &format!("toasts-{b}-close")));
    input.key(&mut tree, 13);
    assert_eq!(t.count(), 1, "Enter closed it");
    frame(&mut tree, 420.0, 400.0);
    input.click(&mut tree, &format!("toasts-{c}-close"));
    assert_eq!(t.count(), 0);
}

#[test]
fn the_pointer_over_a_toast_holds_its_time() {
    let _g = serial();
    let t = stack();
    let id = t.push_at(Toast::new(Severity::Info, "a").lasts(10), 0);
    let mut tree = layout(t.build(), 420.0, 200.0);
    let mut input = Input::new();
    let (x, y) = Input::centre(&tree, &format!("toasts-{id}"));
    input.move_to(&mut tree, x, y);
    assert_eq!(
        t.tick(common_now() + 10_000),
        0,
        "held while the pointer is over it"
    );
    input.move_to(&mut tree, 419.0, 199.0);
    assert_eq!(t.tick(common_now() + 20_000), 1, "released");
}

fn common_now() -> u64 {
    fs_components_widgets::toast::now_ms()
}

#[test]
fn roles_tell_what_interrupts() {
    assert_eq!(Severity::Error.role(), "alert");
    assert_eq!(Severity::Warn.role(), "alert");
    assert_eq!(Severity::Info.role(), "status");
    assert_eq!(Severity::Ok.role(), "status");
    let glyphs: std::collections::HashSet<_> = Severity::ALL.iter().map(|s| s.glyph()).collect();
    assert_eq!(
        glyphs.len(),
        4,
        "four kinds, four glyphs: not told by colour alone"
    );
}
