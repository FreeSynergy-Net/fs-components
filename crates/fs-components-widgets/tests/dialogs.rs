//! The dialogs: words in two languages, states, keys (Escape, Enter, Tab, arrows), the Tab trap and the buttons.
//!
//! Tests: Model tests over the words, an in-process mouse and keyboard over the built dialogs (the window's own router).

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use common::{Input, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::dialogs::{
    ChoiceDialog, ChoiceOption, ConfirmDialog, Labels, LicenseDialog, TEXTS, modal,
};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<String>>>;

fn say(log: &Log, what: &str) {
    log.lock().unwrap().push(what.to_owned());
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn confirm(log: &Log, danger: bool) -> ConfirmDialog {
    let (a, b) = (Arc::clone(log), Arc::clone(log));
    ConfirmDialog::new(&Labels::default(), "Remove Tasks?", "The data stays.")
        .id("cd")
        .danger(danger)
        .on_confirm(move || say(&a, "yes"))
        .on_cancel(move || say(&b, "no"))
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(
        (de.accept.as_str(), de.decline.as_str()),
        ("Zustimmen", "Ablehnen")
    );
    assert_eq!(Labels::default().cancel, "Cancel");
}

#[test]
fn confirm_enter_takes_it_escape_leaves() {
    let _g = serial();
    let log = Log::default();
    let d = confirm(&log, false);
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    input.key(&mut tree, 27);
    assert_eq!(seen(&log), ["yes", "no"]);
}

#[test]
fn a_danger_confirm_does_not_take_enter_alone_but_the_button() {
    let _g = serial();
    let log = Log::default();
    let d = confirm(&log, true);
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    assert!(
        seen(&log).is_empty(),
        "Enter with no button focused did something"
    );
    input.click(&mut tree, "cd-ok");
    input.click(&mut tree, "cd-cancel");
    assert_eq!(seen(&log), ["yes", "no"]);
}

#[test]
fn enter_on_a_focused_button_is_that_button() {
    let _g = serial();
    let log = Log::default();
    let d = confirm(&log, false);
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cd-cancel"));
    input.key(&mut tree, 13);
    assert_eq!(
        seen(&log),
        ["no"],
        "Enter on Cancel must cancel, not confirm"
    );
}

#[test]
fn a_busy_confirm_answers_nothing() {
    let _g = serial();
    let log = Log::default();
    let d = confirm(&log, false);
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    d.set_busy(true);
    common::frame(&mut tree, 480.0, 260.0);
    assert!(d.is_busy());
    input.key(&mut tree, 13);
    input.key(&mut tree, 27);
    assert!(!d.confirm() && !d.cancel());
    assert_eq!(seen(&log), Vec::<String>::new());
    d.set_busy(false);
    assert!(d.confirm());
}

fn choice(log: &Log) -> ChoiceDialog {
    let (a, b) = (Arc::clone(log), Arc::clone(log));
    ChoiceDialog::new(&Labels::default(), "Which font?")
        .id("ch")
        .message("The theme needs a font.")
        .option(ChoiceOption::new("mono", "JetBrains Mono").hint("monospace"))
        .option(ChoiceOption::new("old", "Old Mono").disabled())
        .option(ChoiceOption::new("sans", "Inter"))
        .on_choose(move |k| say(&a, &format!("chose {k}")))
        .on_cancel(move || say(&b, "cancel"))
}

#[test]
fn choice_starts_on_the_first_option_that_can_be_picked_and_arrows_skip_disabled() {
    let _g = serial();
    let log = Log::default();
    let d = choice(&log);
    assert_eq!(d.picked().as_deref(), Some("mono"));
    let mut tree = layout(d.build(), 480.0, 360.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "ch-list"));
    input.key(&mut tree, 40);
    assert_eq!(
        d.picked().as_deref(),
        Some("sans"),
        "the disabled option is skipped"
    );
    input.key(&mut tree, 40);
    assert_eq!(d.picked().as_deref(), Some("sans"), "no wrap past the end");
    input.key(&mut tree, 36);
    assert_eq!(d.picked().as_deref(), Some("mono"));
    input.key(&mut tree, 35);
    assert_eq!(d.picked().as_deref(), Some("sans"));
    input.key(&mut tree, 38);
    assert_eq!(d.picked().as_deref(), Some("mono"));
}

#[test]
fn choice_enter_takes_the_pick_the_pointer_picks_and_escape_leaves() {
    let _g = serial();
    let log = Log::default();
    let d = choice(&log);
    let mut tree = layout(d.build(), 480.0, 360.0);
    let mut input = Input::new();
    input.click(&mut tree, "ch-opt-old");
    assert_eq!(
        d.picked().as_deref(),
        Some("mono"),
        "a disabled option ignores the pointer"
    );
    input.click(&mut tree, "ch-opt-sans");
    assert_eq!(d.picked().as_deref(), Some("sans"));
    input.key(&mut tree, 13);
    input.click(&mut tree, "ch-ok");
    input.key(&mut tree, 27);
    assert_eq!(seen(&log), ["chose sans", "chose sans", "cancel"]);
}

#[test]
fn a_choice_with_nothing_to_pick_takes_nothing() {
    let _g = serial();
    let d = ChoiceDialog::new(&Labels::default(), "None")
        .option(ChoiceOption::new("a", "A").disabled());
    assert!(d.picked().is_none() && !d.choose() && !d.select("a"));
}

fn scroll_frame(tree: &mut blinc_layout::prelude::RenderTree) {
    tree.process_pending_scroll_refs();
    tree.tick_scroll_physics(1_000);
}

const LICENCE: &str = "Copyright (c) 2024 Someone\n\nPermission is hereby granted, free of charge,\n  to any person obtaining a copy of this software.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\".";

fn licence(log: &Log) -> LicenseDialog {
    let (a, b, c) = (Arc::clone(log), Arc::clone(log), Arc::clone(log));
    LicenseDialog::new(&Labels::default(), "Licence", "MIT", LICENCE)
        .id("ld")
        .on_accept(move || say(&a, "accept"))
        .on_decline(move || say(&b, "decline"))
        .on_copy(move |t| say(&c, &format!("copy {}", t.len())))
}

#[test]
fn the_licence_text_is_shown_as_given_and_copied_whole() {
    let _g = serial();
    let log = Log::default();
    let d = licence(&log);
    assert_eq!(d.text(), LICENCE, "not a character changed");
    let mut tree = layout(d.build(), 520.0, 520.0);
    let mut input = Input::new();
    input.click(&mut tree, "ld-copy");
    assert_eq!(seen(&log), [format!("copy {}", LICENCE.len())]);
}

#[test]
fn the_licence_is_not_accepted_by_enter_alone_but_escape_declines() {
    let _g = serial();
    let log = Log::default();
    let d = licence(&log);
    let mut tree = layout(d.build(), 520.0, 520.0);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    assert_eq!(seen(&log), Vec::<String>::new());
    input.key(&mut tree, 27);
    input.click(&mut tree, "ld-accept");
    input.click(&mut tree, "ld-decline");
    assert_eq!(seen(&log), ["decline", "accept", "decline"]);
}

#[test]
fn a_long_licence_scrolls_by_wheel_and_keys() {
    let _g = serial();
    let log = Log::default();
    let text = (0..120)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let d = LicenseDialog::new(&Labels::default(), "Licence", "Long", &text).id("ld");
    let mut tree = layout(d.build(), 520.0, 520.0);
    let mut input = Input::new();
    assert_eq!(d.scrolled(), 0.0);
    let (x, y) = Input::centre(&tree, "ld-text");
    input.wheel_at(&mut tree, x, y, 120.0);
    let by_wheel = d.scrolled();
    assert!(by_wheel > 0.0, "the wheel did not scroll ({by_wheel})");
    assert!(input.tab_to(&mut tree, "ld-text"), "the text is a tab stop");
    input.key(&mut tree, 34);
    scroll_frame(&mut tree); // what a window does every frame
    common::frame(&mut tree, 520.0, 520.0);
    assert!(d.scrolled() > by_wheel, "PageDown did not scroll");
    input.key(&mut tree, 36);
    scroll_frame(&mut tree);
    common::frame(&mut tree, 520.0, 520.0);
    assert!(
        d.scrolled() < 1.0,
        "Home did not go to the top: {}",
        d.scrolled()
    );
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn tab_stays_in_the_dialog_and_starts_again_at_the_first_stop() {
    let _g = serial();
    let log = Log::default();
    let d = confirm(&log, false);
    let page = div()
        .child(div().id("page-btn").attr("tabindex", "0").w(80.0).h(30.0))
        .child(div().id("page-btn-2").attr("tabindex", "0").w(80.0).h(30.0));
    let mut tree = layout(modal(page, d.build()), 640.0, 480.0);
    let mut input = Input::new();
    assert!(
        input.tab_to(&mut tree, "cd-cancel"),
        "Tab reaches the dialog"
    );
    assert!(input.tab_to(&mut tree, "cd-ok"));
    assert!(
        input.tab_to(&mut tree, "cd-cancel"),
        "Tab wraps to the first stop"
    );
    assert!(
        !input.tab_to(&mut tree, "page-btn"),
        "the page behind took the keyboard"
    );
    assert!(!input.tab_to(&mut tree, "page-btn-2"));
}
