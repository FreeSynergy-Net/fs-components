//! `carousel`: region role and role description, slide group with its position name, buttons, Left/Right/Home/End (window keys AND raw),
//! wrap and no wrap, Enter and Space are not carousel keys, the buttons press once (click, window click, raw keys), the words in two
//! languages, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, `frame` between presses as a window does.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use blinc_layout::prelude::{RenderTree, text};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::carousel::{Carousel, Labels, STYLES, State, TEXTS};
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<usize>>>;

fn carousel(log: &Log, wrap: bool) -> Carousel {
    let l = Arc::clone(log);
    let c = Carousel::new("Featured", &Labels::default())
        .id("cr")
        .slide(|| div().child(text("one")))
        .slide(|| div().child(text("two")))
        .slide(|| div().child(text("three")))
        .on_show(move |i| l.lock().unwrap().push(i));
    if wrap { c } else { c.no_wrap() }
}

fn show(c: &Carousel) -> RenderTree {
    layout_with(c.build(), 500.0, 300.0, STYLES)
}

fn seen(log: &Log) -> Vec<usize> {
    log.lock().unwrap().clone()
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(
        (de.kind.as_str(), de.slide.as_str()),
        ("Karussell", "Folie")
    );
    assert_eq!(
        (de.previous.as_str(), de.next.as_str()),
        ("Vorherige Folie", "Nächste Folie")
    );
    let en = Labels::default();
    assert_eq!(
        (
            en.kind.as_str(),
            en.slide.as_str(),
            en.previous.as_str(),
            en.next.as_str()
        ),
        ("carousel", "slide", "Previous slide", "Next slide")
    );
}

#[test]
fn the_root_is_a_named_region_the_slide_a_group_with_its_place_and_the_buttons_are_named() {
    let _g = serial();
    let tree = show(&carousel(&Log::default(), true));
    assert_eq!(attr(&tree, "cr", "role").as_deref(), Some("region"));
    assert_eq!(
        attr(&tree, "cr", "aria-roledescription").as_deref(),
        Some("carousel")
    );
    assert_eq!(attr(&tree, "cr", "aria-label").as_deref(), Some("Featured"));
    assert_eq!(attr(&tree, "cr", "tabindex").as_deref(), Some("0"));
    assert_eq!(attr(&tree, "cr-slide", "role").as_deref(), Some("group"));
    assert_eq!(
        attr(&tree, "cr-slide", "aria-roledescription").as_deref(),
        Some("slide")
    );
    assert_eq!(
        attr(&tree, "cr-slide", "aria-label").as_deref(),
        Some("1 / 3")
    );
    assert_eq!(
        attr(&tree, "cr-slide", "aria-live").as_deref(),
        Some("polite")
    );
    assert_eq!(
        attr(&tree, "cr-previous", "role").as_deref(),
        Some("button")
    );
    assert_eq!(
        attr(&tree, "cr-previous", "aria-label").as_deref(),
        Some("Previous slide")
    );
    assert_eq!(
        attr(&tree, "cr-next", "aria-label").as_deref(),
        Some("Next slide")
    );
}

#[test]
fn left_right_home_end_show_slides_with_a_frame_between_and_wrap() {
    let _g = serial();
    let log = Log::default();
    let c = carousel(&log, true);
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cr"));
    for (code, want) in [(39, 1), (39, 2), (39, 0), (37, 2), (36, 0), (35, 2)] {
        input.key(&mut tree, code);
        frame(&mut tree, 500.0, 300.0);
        assert_eq!(c.current(), want, "key {code}");
        assert!(
            input.has_keyboard(&tree, "cr"),
            "the carousel keeps the keyboard"
        );
        assert_eq!(
            attr(&tree, "cr-slide", "aria-label"),
            Some(format!("{} / 3", want + 1))
        );
    }
    assert_eq!(seen(&log), [1, 2, 0, 2, 0, 2]);
    input.key(&mut tree, 35);
    assert_eq!(seen(&log).len(), 6, "End on the last slide says nothing");
}

#[test]
fn without_wrap_the_ends_stop_and_other_keys_do_nothing() {
    let _g = serial();
    let log = Log::default();
    let c = carousel(&log, false);
    let mut tree = show(&c);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "cr"));
    input.key(&mut tree, 37);
    assert_eq!(c.current(), 0, "no wrap back");
    input.key(&mut tree, 35);
    input.key(&mut tree, 39);
    assert_eq!(c.current(), 2, "no wrap forward");
    for code in [38, 40, 13, 32] {
        input.key(&mut tree, code);
    }
    input.key_event_to(&mut tree, "cr", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "cr", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "cr", KEY_DOWN, 39);
    assert_eq!(
        c.current(),
        2,
        "Up, Down, Enter and Space are not carousel keys"
    );
    input.key_event_to(&mut tree, "cr", KEY_DOWN, 37);
    assert_eq!(c.current(), 1, "a raw Left goes back");
    assert_eq!(seen(&log), [2, 1]);
}

#[test]
fn the_buttons_press_once_by_click_window_click_and_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let c = carousel(&log, true);
    let mut tree = show(&c);
    let mut input = Input::new();
    input.click(&mut tree, "cr-next");
    frame(&mut tree, 500.0, 300.0);
    assert_eq!(seen(&log), [1], "a click presses once");
    assert!(input.tab_to(&mut tree, "cr-next"));
    input.key(&mut tree, 13);
    assert_eq!(
        seen(&log),
        [1, 2],
        "Enter (the window's click) presses once"
    );
    input.key_event_to(&mut tree, "cr-previous", KEY_DOWN, 32);
    assert_eq!(
        c.current(),
        2,
        "the previous button is not focused: a raw key to it does nothing"
    );
    assert!(input.tab_to(&mut tree, "cr-previous"));
    input.key_event_to(&mut tree, "cr-previous", KEY_DOWN, 32);
    assert_eq!(c.current(), 1, "a raw Space on the focused previous button");
    input.key_event_to(&mut tree, "cr-previous", KEY_DOWN, 65);
    assert_eq!(c.current(), 1);
}

#[test]
fn the_model_holds_its_edges_and_an_empty_carousel_does_nothing() {
    let mut s = State::new(3, true);
    assert!(s.wraps() && s.go(2) && !s.go(2) && !s.go(3));
    let mut e = State::new(0, true);
    for k in [
        fs_components_widgets::Key::Right,
        fs_components_widgets::Key::Left,
        fs_components_widgets::Key::Home,
        fs_components_widgets::Key::End,
    ] {
        assert_eq!(
            e.press(k),
            fs_components_widgets::carousel::Outcome::Ignored
        );
    }
    let one = Carousel::new("x", &Labels::default()).id("c1").slide(div);
    one.go(5);
    assert_eq!(one.current(), 0);
    assert!(
        !one.next(),
        "a single slide wraps onto itself: nothing to show"
    );
}
