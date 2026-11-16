//! The surfaces of the four states (empty, loading, failed, partial) and the blocks under them.
//!
//! Tests: Model tests over the words, in-process pointer over the built surfaces (the retry and the action buttons).

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use blinc_layout::prelude::text;
use common::{Input, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::icon::Glyph;
use fs_components_widgets::states::{
    Alert, EmptyState, Labels, Load, PartialNotice, Severity, Skeleton, Surface, TEXTS,
};
use fs_components_words::{Locale, Localized};

fn surface(log: &Arc<Mutex<Vec<&'static str>>>) -> Surface {
    let (a, b) = (Arc::clone(log), Arc::clone(log));
    let empty = EmptyState::new("No appointments")
        .glyph(Glyph::Calendar)
        .text("Nothing in this period.")
        .action("Create appointment", move || {
            a.lock().unwrap().push("create");
        });
    Surface::new(&Labels::default(), empty)
        .id("st")
        .on_retry(move || b.lock().unwrap().push("retry"))
}

fn ready() -> blinc_layout::div::Div {
    div().child(text("content").id("content"))
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.retry, "Erneut versuchen");
    assert_eq!(de.partial(2, 9), "2 von 9 konnten nicht gezeigt werden");
    assert_eq!(Labels::default().partial(2, 9), "2 of 9 could not be shown");
}

#[test]
fn each_state_shows_its_own_surface_and_only_ready_and_partial_show_the_content() {
    let _g = serial();
    let log = Arc::default();
    let s = surface(&log);
    let partial = Load::Partial {
        missing: vec!["Work".into(), "Holidays".into()],
        total: 9,
    };
    let cases = [
        (Load::Loading, "st-skeleton", false),
        (Load::Empty, "st-empty", false),
        (
            Load::Failed("The server did not answer.".into()),
            "st-failed",
            false,
        ),
        (partial, "st-partial", true),
        (Load::Ready, "content", true),
    ];
    for (load, part, content) in cases {
        let tree = layout(s.build(&load, ready), 480.0, 400.0);
        assert!(tree.query_by_id(part).is_some(), "{load:?}: no {part}");
        assert_eq!(tree.query_by_id("content").is_some(), content, "{load:?}");
    }
}

#[test]
fn only_one_surface_at_a_time() {
    let _g = serial();
    let log = Arc::default();
    let s = surface(&log);
    let tree = layout(s.build(&Load::Empty, ready), 480.0, 400.0);
    for other in ["st-skeleton", "st-failed", "st-partial"] {
        assert!(
            tree.query_by_id(other).is_none(),
            "{other} next to the empty surface"
        );
    }
}

#[test]
fn the_failed_surface_retries_the_empty_one_acts_the_partial_one_retries() {
    let _g = serial();
    let log = Arc::default();
    let s = surface(&log);
    let mut input = Input::new();
    let mut tree = layout(s.build(&Load::Failed("no".into()), ready), 480.0, 300.0);
    input.click(&mut tree, "st-failed-action");
    let mut tree = layout(s.build(&Load::Empty, ready), 480.0, 300.0);
    input.click(&mut tree, "st-empty-action");
    let load = Load::Partial {
        missing: vec!["Work".into()],
        total: 3,
    };
    let mut tree = layout(s.build(&load, ready), 480.0, 300.0);
    input.click(&mut tree, "st-partial-action");
    assert_eq!(*log.lock().unwrap(), ["retry", "create", "retry"]);
}

#[test]
fn without_a_retry_the_failed_surface_has_no_dead_button() {
    let _g = serial();
    let s = Surface::new(&Labels::default(), EmptyState::new("None")).id("st");
    let tree = layout(s.build(&Load::Failed("no".into()), ready), 480.0, 300.0);
    assert!(tree.query_by_id("st-failed-action").is_none());
}

#[test]
fn the_partial_notice_says_how_many_and_lists_what_is_missing() {
    let _g = serial();
    let miss = vec!["Work".to_owned(), "Holidays".to_owned()];
    let n = PartialNotice::new(&Labels::default(), &miss, 9).id("pn");
    assert_eq!(n.lead(), "2 of 9 could not be shown");
    let tree = layout(n.build(), 480.0, 200.0);
    assert!(tree.query_by_id("pn-text").is_some());
}

#[test]
fn the_skeleton_is_busy_and_holds_the_shape() {
    let _g = serial();
    let s = Skeleton::new("Loading")
        .id("sk")
        .lines(3)
        .rows(2)
        .block(100.0, 60.0);
    assert_eq!(s.bars(), 3 + 4 + 1);
    let tree = layout(s.build(), 400.0, 400.0);
    for id in ["sk", "sk-bar-0-0", "sk-bar-2-0", "sk-bar-3-1", "sk-bar-5-0"] {
        assert!(tree.query_by_id(id).is_some(), "{id}");
    }
    let (w, h) = common::size_of(&tree, "sk-bar-5-0");
    assert_eq!((w, h), (100.0, 60.0));
}

#[test]
fn an_alert_has_the_four_steps_and_the_failed_form() {
    let _g = serial();
    for step in Severity::ALL {
        let tree = layout(Alert::new(step, "text").id("al").build(), 400.0, 120.0);
        assert!(tree.query_by_id("al-text").is_some(), "{step:?}");
    }
    let failed = Alert::failed(&Labels::default(), "Timed out", || {}).id("al");
    assert_eq!(failed.severity(), Severity::Error);
    let tree = layout(failed.build(), 400.0, 120.0);
    assert!(tree.query_by_id("al-title").is_some() && tree.query_by_id("al-action").is_some());
}
